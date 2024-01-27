use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
    time::Duration,
};

use crate::{
    dao::{
        MailTaskData, MailTaskItem, MessageTpls, SenderError, SenderExecError, SenderResult,
        SenderTaskExecutor, SenderTaskResult, SenderTaskResultItem, SenderTaskStatus,
        SenderTplConfig,
    },
    model::{SenderTplConfigModel, SenderType},
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
use lsys_core::{fluent_message, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_setting::{
    dao::{
        MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingJson, SettingKey,
        SettingResult,
    },
    model::SettingModel,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tera::Context;
use tokio::sync::RwLock;

use sqlx::Pool;
use tracing::debug;

// 邮件发送 smtp 适配

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub user: String,
    pub email: String,
    pub password: String,
    pub tls_domain: String,
    pub branch_limit: u16,
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

impl SettingKey for SmtpConfig {
    fn key<'t>() -> &'t str {
        "smtp-config"
    }
}

impl SettingDecode for SmtpConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for SmtpConfig {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for SmtpConfig {}

#[derive(Serialize, Deserialize, Default)]
pub struct SmtpTplConfig {
    pub from_email: String,
    pub reply_email: String,
    pub subject_tpl_id: String,
    pub body_tpl_id: String,
}

pub fn check_email(email: &str) -> SenderResult<()> {
    let re = Regex::new(r"^[A-Za-z0-9\u4e00-\u9fa5\.\-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$")
        .map_err(|e| {
            SenderError::System(lsys_core::fluent_message!("check-email-error",
                {
                    "mail":email,
                    "msg":e
                }
            ))
        })?;
    if !re.is_match(email) {
        return Err(SenderError::System(
            lsys_core::fluent_message!("check-email-not-match",
                {
                    "mail":email,
                }

            ),
        )); //  "submit email is invalid"
    }
    Ok(())
}

//邮件发送smtp配置
pub struct SenderSmtpConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderSmtpConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            tpl_config,
            setting,
        }
    }
    //列出有效的smtp配置
    pub async fn list_config(
        &self,
        config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<SettingData<SmtpConfig>>> {
        Ok(self
            .setting
            .list_data::<SmtpConfig>(&None, config_ids, &None)
            .await?)
    }
    //删除指定的smtp配置
    pub async fn del_config(
        &self,
        id: &u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<SmtpConfig>(&None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的smtp配置
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        config: &SmtpConfig,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.check_config(config).await?;
        Ok(self
            .setting
            .edit(&None, id, name, config, user_id, None, env_data)
            .await?)
    }
    //添加smtp配置
    pub async fn add_config(
        &self,
        name: &str,
        config: &SmtpConfig,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.check_config(config).await?;
        Ok(self
            .setting
            .add(&None, name, config, user_id, None, env_data)
            .await?)
    }
    //检测smtp配置
    pub async fn check_config(&self, config: &SmtpConfig) -> SenderResult<()> {
        connect(config, 5)
            .await
            .map_err(|e| SenderError::System(fluent_message!("smtp-check-error", e)))?
            .test_connection()
            .await
            .map_err(|e| SenderError::System(fluent_message!("smtp-check-error", e)))?;
        Ok(())
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
        reply_email: &str,
        subject_tpl_id: &str,
        body_tpl_id: &str,
        user_id: &u64,
        add_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.setting
            .load::<SmtpConfig>(&None, smtp_config_id)
            .await?;
        check_email(from_email)?;
        if !reply_email.is_empty() {
            check_email(reply_email)?;
        }
        let from_email = from_email.to_owned();
        let reply_email = reply_email.to_owned();
        let subject_tpl_id = subject_tpl_id.to_owned();
        let body_tpl_id = body_tpl_id.to_owned();
        let res = self
            .tpl_config
            .add_config(
                name,
                app_id,
                smtp_config_id,
                tpl_id,
                &SmtpTplConfig {
                    from_email,
                    reply_email,
                    subject_tpl_id,
                    body_tpl_id,
                },
                user_id,
                add_user_id,
                env_data,
            )
            .await?;

        Ok(res)
    }
}

//邮件发送后台发送任务实现
#[derive(Clone)]
pub struct SmtpSenderTask {
    mailer: Arc<RwLock<HashMap<u64, AsyncSmtpTransport<Tokio1Executor>>>>,
    tpls: Arc<MessageTpls>,
}

impl SmtpSenderTask {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLogger>) -> Self {
        Self {
            mailer: Arc::new(RwLock::new(HashMap::new())),
            tpls: Arc::new(MessageTpls::new(db, logger)),
        }
    }
}

#[async_trait]
impl SenderTaskExecutor<u64, MailTaskItem, MailTaskData> for SmtpSenderTask {
    fn setting_key(&self) -> String {
        SmtpConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<SmtpConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    1
                } else {
                    e.branch_limit
                }
            })
            .ok()
            .unwrap_or(1)
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &MailTaskItem,
        data: &MailTaskData,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> SenderTaskResult {
        debug!("msgid:{} config_id:{} ", val.mail.id, tpl_config.id,);
        let smtp_setting =
            SettingData::<SmtpConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!("parse config to smtp setting fail:{}", e))
            })?;
        let hand_id = format!("{}-{}", smtp_setting.host, smtp_setting.user);
        let mail_tpl_config = serde_json::from_str::<SmtpTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to smtp tpl config fail[{}]:{}",
                    hand_id, e
                ))
            })?;
        let var_tpl = serde_json::from_str::<Value>(&val.mail.tpl_var)
            .map_err(|e| SenderExecError::Finish(format!("load tpl fail[{}]:{}", hand_id, e)))?;
        let context = Context::from_value(var_tpl)
            .map_err(|e| SenderExecError::Finish(format!("prare tpl fail[{}]:{}", hand_id, e)))?;

        let mut bad_res = vec![];
        let mut email_builder = Message::builder();

        if let Ok(from) = mail_tpl_config.from_email.parse::<Mailbox>() {
            email_builder = email_builder.from(from);
        }
        if !val.mail.reply_mail.is_empty() {
            if let Ok(reply_mail) = val.mail.reply_mail.parse::<Mailbox>() {
                email_builder = email_builder.reply_to(reply_mail);
            }
        } else if let Ok(reply) = mail_tpl_config.reply_email.parse::<Mailbox>() {
            email_builder = email_builder.reply_to(reply);
        }

        let mut is_send = false;
        for tmp in data.data.iter() {
            email_builder = match tmp.to_mail.parse::<Mailbox>() {
                Ok(dat) => {
                    is_send = true;
                    email_builder.to(dat)
                }
                Err(err) => {
                    bad_res.push(SenderTaskResultItem {
                        id: tmp.id,
                        status: SenderTaskStatus::Failed(false),
                        message: err.to_string(),
                        send_id: "".to_owned(),
                    });
                    continue;
                }
            };
        }

        let subject = self
            .tpls
            .render(
                SenderType::Mailer,
                &mail_tpl_config.subject_tpl_id,
                &context,
            )
            .await
            .map_err(|e| {
                SenderExecError::Finish(format!("render subject fail [{}]: {}", hand_id, e))
            })?;
        email_builder = email_builder.subject(subject);

        let body = self
            .tpls
            .render(SenderType::Mailer, &mail_tpl_config.body_tpl_id, &context)
            .await
            .map_err(|e| {
                SenderExecError::Finish(format!("render body fail[{}]: {}", hand_id, e))
            })?;

        let msg = email_builder
            .multipart(
                MultiPart::alternative() // This is composed of two parts.
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(body),
                    ),
            )
            .map_err(|e| {
                SenderExecError::Finish(format!("parse mail body fail[{}]: {}", hand_id, e))
            })?;

        if is_send {
            let res = match self.mailer.write().await.entry(tpl_config.id) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    entry.insert(connect(&smtp_setting, smtp_setting.timeout).await.map_err(
                        |e| SenderExecError::Next(format!("connect fail{}: {}", hand_id, e)),
                    )?)
                }
            }
            .send(msg)
            .await;
            let mut send_res = match res {
                Ok(response) => data
                    .data
                    .iter()
                    .map(|e| SenderTaskResultItem {
                        id: e.id,
                        status: SenderTaskStatus::Completed,
                        message: response.message().collect::<Vec<&str>>().join(","),
                        send_id: "".to_owned(),
                    })
                    .collect::<Vec<_>>(),
                Err(err) => data
                    .data
                    .iter()
                    .map(|e| SenderTaskResultItem {
                        id: e.id,
                        status: SenderTaskStatus::Failed(true),
                        message: format!("send email fail: {}", err),
                        send_id: "".to_owned(),
                    })
                    .collect::<Vec<_>>(),
            };
            send_res.extend(bad_res);
            Ok(send_res)
        } else {
            Ok(bad_res)
        }
    }
}

async fn connect(
    config: &SmtpConfig,
    timeout: u64,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let mut mailer_builder = AsyncSmtpTransport::<Tokio1Executor>::relay(config.host.as_str())
        .map_err(|e| e.to_string())?;
    if !config.user.is_empty() || !config.password.is_empty() {
        let creds = Credentials::new(config.user.clone(), config.password.clone());
        mailer_builder = mailer_builder.credentials(creds)
    }
    if timeout > 0 {
        mailer_builder = mailer_builder.timeout(Some(Duration::from_secs(timeout)));
    }
    if !config.tls_domain.is_empty() {
        let tls = TlsParametersBuilder::new(config.tls_domain.clone())
            .build()
            .map_err(|e| e.to_string())?;
        mailer_builder = mailer_builder.tls(Tls::Required(tls))
    }
    Ok(mailer_builder.build())
}
