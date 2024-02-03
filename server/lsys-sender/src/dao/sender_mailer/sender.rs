use std::{collections::HashSet, sync::Arc};

use lsys_core::{fluent_message, now_time, AppCore, FluentMessage, RequestEnv, TaskData};

use lsys_logger::dao::ChangeLogger;
use lsys_setting::dao::Setting;
use sqlx::{MySql, Pool};
use tracing::warn;

use super::{MailRecord, MailTaskAcquisition, MailTaskData, MailTaskItem, MailerTask};
use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
        SenderTaskExecutor, SenderTplConfig, SenderWaitItem, SenderWaitNotify,
    },
    model::{SenderMailBodyModel, SenderMailMessageModel, SenderType},
};
use lsys_core::TaskDispatch;

const MAILER_REDIS_PREFIX: &str = "sender-mail-";

pub struct MailSender {
    pub tpl_config: Arc<SenderTplConfig>,
    pub mail_record: Arc<MailRecord>,
    redis: deadpool_redis::Pool,
    db: Pool<sqlx::MySql>,
    app_core: Arc<AppCore>,
    message_logs: Arc<MessageLogs>,
    cancel: Arc<MessageCancel>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
    task: TaskDispatch<u64, MailTaskItem>,
    send_wait: Arc<SenderWaitNotify>,
}

impl MailSender {
    //发送
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        setting: Arc<Setting>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        wait_timeout: Option<u8>,
    ) -> Self {
        let config: Arc<SenderConfig> = Arc::new(SenderConfig::new(
            db.clone(),
            logger.clone(),
            SenderType::Mailer,
        ));
        let tpl_config = Arc::new(SenderTplConfig::new(
            db.clone(),
            setting,
            logger.clone(),
            SenderType::Mailer,
        ));
        let cancel = Arc::new(MessageCancel::new(db.clone(), SenderType::Mailer));
        let message_logs = Arc::new(MessageLogs::new(db.clone(), SenderType::Mailer));
        let message_reader = Arc::new(MessageReader::new(
            db.clone(),
            app_core.clone(),
            SenderType::Mailer,
        ));
        let mail_record = Arc::new(MailRecord::new(
            db.clone(),
            config,
            logger,
            message_logs.clone(),
            message_reader.clone(),
        ));
        let wait_status_key = format!("{}-status-data", MAILER_REDIS_PREFIX);
        let send_wait = Arc::new(SenderWaitNotify::new(
            &wait_status_key,
            redis.clone(),
            app_core.clone(),
            wait_timeout.unwrap_or(30),
        ));

        let task = TaskDispatch::new(
            format!("{}-notify", MAILER_REDIS_PREFIX),
            format!("{}-read-lock", MAILER_REDIS_PREFIX),
            format!("{}-run-task", MAILER_REDIS_PREFIX),
            task_size,
            task_timeout,
            is_check,
            task_timeout,
        );
        Self {
            tpl_config,
            redis,
            mail_record,
            app_core,
            db,
            message_logs,
            message_reader,
            task,
            send_wait,
            cancel,
        }
    }
    //发送模板消息
    #[allow(clippy::too_many_arguments)]
    pub async fn send<'t>(
        &self,
        app_id: Option<u64>,
        mail: &[&'t str],
        tpl_id: &str,
        tpl_var: &str,
        send_time: &Option<u64>,
        user_id: &Option<u64>,
        reply_mail: &Option<String>,
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str, Result<bool, FluentMessage>)>)> {
        let tmp = mail
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .copied()
            .collect::<Vec<_>>();
        let nt = now_time().unwrap_or_default();
        let sendtime = send_time.unwrap_or(nt);
        let sendtime = if sendtime < nt { nt } else { sendtime };
        let max_try_num = max_try_num.unwrap_or(1);
        self.mail_record
            .send_check(app_id, tpl_id, &tmp, sendtime)
            .await?;
        let res = self
            .mail_record
            .add(
                &tmp,
                &app_id.unwrap_or_default(),
                tpl_id,
                tpl_var,
                &sendtime,
                reply_mail,
                user_id,
                &max_try_num,
                env_data,
            )
            .await?;

        let mut wait = None;
        if max_try_num == 0 && mail.len() == 1 {
            if let Some((msg_id, _)) = res.1.first() {
                wait = Some(self.send_wait.wait(SenderWaitItem(res.0, *msg_id)).await);
            }
        };

        if send_time
            .map(|e| e - 1 <= now_time().unwrap_or_default())
            .unwrap_or(true)
        {
            let mut redis = self.redis.get().await?;
            if let Err(err) = self.task.notify(&mut redis).await {
                warn!("mail is add [{}] ,but send fail :{}", res.0, err)
            }
        }

        let mut tmp = vec![];
        for e in mail {
            let msg_id = res
                .1
                .iter()
                .find(|t| t.1 == *e)
                .map(|e| e.0)
                .unwrap_or_default();
            let item_res = if let Some(t) = wait.take() {
                self.send_wait
                    .wait_timeout(t)
                    .await
                    .map(|e| e.map_err(|c| fluent_message!("mail-send-fail", c)))
                    .unwrap_or_else(|e| Err(fluent_message!("mail-send-wait-fail", e)))
            } else {
                Ok(true)
            };
            tmp.push((msg_id, *e, item_res));
        }
        Ok((res.0, tmp))
    }
    //通过ID取消发送
    pub async fn cancal_from_message(
        &self,
        body: &SenderMailBodyModel,
        msg_data: &[&SenderMailMessageModel],
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        self.cancel
            .add(
                &body.app_id,
                &body.id,
                &msg_data.iter().map(|e| e.id).collect::<Vec<_>>(),
                user_id,
                None,
            )
            .await?;

        let mut out = Vec::with_capacity(msg_data.len());
        for (msg, task_data) in self
            .task_is_run(msg_data.iter().map(|e| (&e.id, *e)).collect::<Vec<_>>())
            .await?
        {
            let err = if task_data.is_none() {
                self.mail_record
                    .cancel_form_message(body, msg, user_id, env_data)
                    .await
                    .err()
            } else {
                Some(SenderError::System(
                    fluent_message!("mail-send-ok-cancel", //  "mail {} is send:{}",
                        {
                            "to_mail":msg.to_mail,
                            "msg_id":msg.id
                        }
                    ),
                ))
            };
            out.push((msg.id, task_data.is_none(), err))
        }
        Ok(out)
    }
    //检查指定任务是否发送中
    pub async fn task_is_run<D>(
        &self,
        check_message_data: Vec<(&u64, D)>,
    ) -> SenderResult<Vec<(D, Option<TaskData>)>> {
        let mut redis = self.redis.get().await?;
        let mut tdata = self.task.task_data(&mut redis).await?;
        let mut out = Vec::with_capacity(check_message_data.len());
        for (mid, data) in check_message_data {
            out.push((data, tdata.remove(mid)));
        }
        Ok(out)
    }
    pub async fn cancal_from_message_snid_vec(
        &self,
        msg_snid_data: &[u64],
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<Vec<(u64, bool, Option<SenderError>)>> {
        let res = self
            .message_reader
            .find_message_by_snid_vec(msg_snid_data)
            .await?;
        if res.is_empty() {
            return Ok(vec![]);
        }
        let b_res_id = res.iter().map(|e| e.sender_body_id).collect::<Vec<_>>();
        let b_res = self.message_reader.find_body_by_id_vec(&b_res_id).await?;
        let mut out = vec![];
        for b_tmp in b_res {
            //查找指定body下的msg
            let tmp = res
                .iter()
                .filter(|c| c.sender_body_id == b_tmp.id)
                .collect::<Vec<_>>();

            out.extend(
                self.cancal_from_message(&b_tmp, &tmp, user_id, env_data)
                    .await?,
            )
        }
        Ok(out)
    }
    //发送等待回调处理监听
    pub async fn task_wait(&self) {
        self.send_wait.listen().await;
    }
    //后台发送任务，内部循环不退出
    pub async fn task_sender(
        &self,
        se: Vec<Box<dyn SenderTaskExecutor<u64, MailTaskItem, MailTaskData>>>,
    ) -> SenderResult<()> {
        let acquisition = Arc::new(MailTaskAcquisition::new(
            self.db.clone(),
            self.send_wait.clone(),
            self.message_logs.clone(),
            self.message_reader.clone(),
        ));
        self.task
            .dispatch(
                self.app_core.clone(),
                acquisition.as_ref(),
                MailerTask::new(acquisition.to_owned(), self.tpl_config.clone(), se)?,
            )
            .await;
        Ok(())
    }
}
