use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{atomic::AtomicU32, Arc},
};

use crate::{
    dao::{AppConfigReader, MessageTpls, SenderError, SenderResult},
    model::{SenderMailSmtpModel, SenderMailSmtpModelRef, SenderMailSmtpStatus, SenderType},
};
use async_trait::async_trait;
use lettre::{
    message::{header, Mailbox, MultiPart, SinglePart},
    transport::smtp::{
        authentication::Credentials,
        client::{Tls, TlsParametersBuilder},
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use lsys_core::{now_time, FluentMessage};
use lsys_setting::dao::{
    MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingError, SettingKey,
    SettingResult,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::Context;
use tokio::sync::RwLock;

use super::{MailTaskItem, MailTaskRecord, MailerTaskExecutioner};
use sqlx::{MySql, Pool};
use sqlx_model::Insert;
use sqlx_model::Update;
use tracing::debug;

async fn connect(config: &SmtpConfig) -> SenderResult<AsyncSmtpTransport<Tokio1Executor>> {
    let mut mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(config.host.as_str())
        .map_err(|e| SenderError::Exec(e.to_string()))?;
    if !config.user.is_empty() || !config.password.is_empty() {
        let creds = Credentials::new(config.user.clone(), config.password.clone());
        mailer_builder = mailer_builder.credentials(creds)
    }
    if !config.tls_domain.is_empty() {
        let tls = TlsParametersBuilder::new(config.tls_domain.clone())
            .build()
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        mailer_builder = mailer_builder.tls(Tls::Required(tls))
    }
    Ok(mailer_builder.build())
}

//smtp  邮件发送

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub user: String,
    pub password: String,
    pub tls_domain: String,
}

impl SmtpConfig {
    pub fn hide_password(&self) -> String {
        let len = self.password.chars().count();
        format!(
            "{}**{}",
            self.password.chars().take(2).collect::<String>(),
            self.password
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
    pub fn hide_user(&self) -> String {
        let len = self.user.chars().count();
        format!(
            "{}**{}",
            self.user.chars().take(2).collect::<String>(),
            self.user
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
}

#[derive(Serialize)]
pub struct ShowSmtpConfig {
    pub id: u64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub user: String,
    pub hide_user: String,
    pub password: String,
    pub hide_password: String,
    pub tls_domain: String,
    pub last_user_id: u64,
    pub last_change_time: u64,
}
impl SettingKey for SmtpConfig {
    fn key<'t>() -> &'t str {
        "smtp-config"
    }
}
impl SettingDecode for SmtpConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_slice::<SmtpConfig>(data.as_bytes())
            .map_err(|e| SettingError::System(e.to_string()))
    }
}

impl SettingEncode for SmtpConfig {
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

pub struct SmtpSender {
    db: Pool<MySql>,
    setting: Arc<MultipleSetting>,
    app_config_read: AppConfigReader<SenderMailSmtpModel, SmtpConfig>,
}

impl SmtpSender {
    pub fn new(db: Pool<sqlx::MySql>, setting: Arc<MultipleSetting>) -> Self {
        Self {
            app_config_read: AppConfigReader::new(db.clone(), setting.clone()),
            db,
            setting,
        }
    }
    //列出有效的smtp配置
    pub async fn list_config(
        &self,
        config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<ShowSmtpConfig>> {
        let data = self
            .setting
            .list_data::<SmtpConfig>(&None, config_ids, &None)
            .await?;
        Ok(data
            .into_iter()
            .map(|e| ShowSmtpConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                last_user_id: e.model().last_user_id,
                last_change_time: e.model().last_change_time,
                host: e.host.clone(),
                port: e.port,
                timeout: e.timeout,
                user: e.user.clone(),
                hide_user: e.hide_user(),
                password: e.password.clone(),
                hide_password: e.hide_password(),
                tls_domain: e.tls_domain.clone(),
            })
            .collect::<Vec<_>>())
    }
    //删除指定的smtp配置
    pub async fn del_config(&self, id: &u64, user_id: &u64) -> SenderResult<u64> {
        Ok(self.setting.del::<SmtpConfig>(&None, id, user_id).await?)
    }
    //编辑指定的smtp配置
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        config: &SmtpConfig,
        user_id: &u64,
    ) -> SenderResult<u64> {
        self.check_config(config).await?;
        Ok(self.setting.edit(&None, id, name, config, user_id).await?)
    }
    //添加smtp配置
    pub async fn add_config(
        &self,
        name: &str,
        config: &SmtpConfig,
        user_id: &u64,
    ) -> SenderResult<u64> {
        self.check_config(config).await?;
        Ok(self.setting.add(&None, name, config, user_id).await?)
    }
    //检测smtp配置
    pub async fn check_config(&self, config: &SmtpConfig) -> SenderResult<()> {
        connect(config).await?;
        Ok(())
    }
    // 配置设置跟app关联
    pub async fn find_app_config_by_id(&self, id: &u64) -> SenderResult<SenderMailSmtpModel> {
        self.app_config_read.find_by_id(id).await
    }
    //查找指定应用的发送跟smtp的配置
    pub async fn find_app_config(
        &self,
        id: &Option<u64>,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
    ) -> SenderResult<Vec<(SenderMailSmtpModel, SettingData<SmtpConfig>)>> {
        self.app_config_read
            .list_config(
                id,
                user_id,
                app_id,
                tpl_id,
                &Some(SenderMailSmtpStatus::Enable as i8),
                None,
                &|e| e.smtp_config_id,
            )
            .await
    }
    //关联发送跟smtp的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: &u64,
        tpl_id: &str,
        smtp_config_id: &u64,
        from_email: &str,
        subject_tpl_id: &str,
        body_tpl_id: &str,
        try_num: &u16,
        user_id: &u64,
        add_user_id: &u64,
    ) -> SenderResult<u64> {
        let name = name.to_owned();
        let tpl_id = tpl_id.to_owned();
        let time = now_time().unwrap_or_default();
        let smtp_config_id = smtp_config_id.to_owned();
        let from_email = from_email.to_owned();
        let subject_tpl_id = subject_tpl_id.to_owned();
        let body_tpl_id = body_tpl_id.to_owned();
        let add = sqlx_model::model_option_set!(SenderMailSmtpModelRef,{
            app_id:app_id,
            name:name,
            tpl_id:tpl_id,
            smtp_config_id:smtp_config_id,
            from_email:from_email,
            subject_tpl_id:subject_tpl_id,
            body_tpl_id:body_tpl_id,
            max_try_num:try_num,
            add_time:time,
            user_id:user_id,
            add_user_id:add_user_id,
            status:SenderMailSmtpStatus::Enable as i8,
        });
        Ok(Insert::<sqlx::MySql, SenderMailSmtpModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())?)
    }
    //删除发送跟smtp的配置
    pub async fn del_app_config(
        &self,
        mail_smtp: &SenderMailSmtpModel,
        user_id: &u64,
    ) -> SenderResult<u64> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderMailSmtpModelRef,{
            status:SenderMailSmtpStatus::Delete as i8,
            delete_time:time,
            delete_user_id:user_id
        });
        let res = Update::<sqlx::MySql, SenderMailSmtpModel, _>::new(change)
            .execute_by_pk(mail_smtp, &self.db)
            .await;
        match res {
            Err(e) => Err(e)?,
            Ok(mr) => {
                //清理缓存
                Ok(mr.rows_affected())
            }
        }
    }
}

#[derive(Clone)]
pub struct SmtpSenderTask {
    smtp: Arc<SmtpSender>,
    i: Arc<AtomicU32>,
    mailer: Arc<RwLock<HashMap<u64, AsyncSmtpTransport<Tokio1Executor>>>>,
    tpls: Arc<MessageTpls>,
}

impl SmtpSenderTask {
    pub fn new(smtp: SmtpSender, db: Pool<sqlx::MySql>, fluent: Arc<FluentMessage>) -> Self {
        Self {
            smtp: Arc::new(smtp),
            i: Arc::new(AtomicU32::new(0)),
            mailer: Arc::new(RwLock::new(HashMap::new())),
            tpls: Arc::new(MessageTpls::new(db, fluent)),
        }
    }
}
#[async_trait]
impl MailerTaskExecutioner<()> for SmtpSenderTask {
    //执行短信发送
    async fn exec(&self, val: MailTaskItem<()>, record: &MailTaskRecord) -> SenderResult<()> {
        let var_tpl = serde_json::from_str::<Value>(&val.mail.tpl_var)
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        let context = Context::from_value(var_tpl).map_err(|e| SenderError::Exec(e.to_string()))?;
        let config = self
            .smtp
            .find_app_config(
                &None,
                &None,
                &Some(val.mail.app_id),
                &Some(val.mail.tpl_id.clone()),
            )
            .await
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        let len = config.len();
        let now = if self.i.load(std::sync::atomic::Ordering::Relaxed) as usize > len {
            self.i.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        } else {
            self.i.store(0, std::sync::atomic::Ordering::Relaxed);
            0
        } as usize;
        let now = if now > len { len } else { now };
        if let Some((mail, config)) = config.get(now) {
            debug!("msgid:{} config_id:{} ", mail.id, config.model().id,);

            let subject = self
                .tpls
                .render(SenderType::Mailer, &mail.subject_tpl_id, &context)
                .await
                .map_err(|e| SenderError::Exec(e.to_string()))?;
            let body = self
                .tpls
                .render(SenderType::Mailer, &mail.body_tpl_id, &context)
                .await
                .map_err(|e| SenderError::Exec(e.to_string()))?;
            let to = val
                .mail
                .to_mail
                .parse::<Mailbox>()
                .map_err(|e| SenderError::Exec(e.to_string()))?;

            let from = mail
                .from_email
                .parse::<Mailbox>()
                .map_err(|e| SenderError::Exec(e.to_string()))?;

            let reply_mail = if val.mail.reply_mail.is_empty() {
                from.clone()
            } else {
                val.mail
                    .reply_mail
                    .parse::<Mailbox>()
                    .map_err(|e| SenderError::Exec(e.to_string()))?
            };
            let email = Message::builder()
                .from(from)
                .reply_to(reply_mail)
                .to(to)
                .subject(subject)
                .multipart(
                    MultiPart::alternative() // This is composed of two parts.
                        .singlepart(
                            SinglePart::builder()
                                .header(header::ContentType::TEXT_HTML)
                                .body(body),
                        ),
                )
                .map_err(|e| SenderError::Exec(e.to_string()))?;
            let res = match self.mailer.write().await.entry(config.model().id) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(connect(config).await?),
            }
            .send(email)
            .await
            .map(|_| {})
            .map_err(|e| e.to_string());

            record
                .finish(
                    "smtp".to_string(),
                    format!("{}-{}", config.host, config.user),
                    &val.mail,
                    &res,
                    mail.max_try_num,
                )
                .await
                .map_err(|e| SenderError::Exec(e.to_string()))?;
            return res.map_err(SenderError::Exec);
        }
        let err = "not find sms config".to_string();
        record
            .finish(
                "smtp".to_string(),
                "".to_string(),
                &val.mail,
                &Err(err.clone()),
                0,
            )
            .await
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        Err(SenderError::Exec(err))
    }
}
