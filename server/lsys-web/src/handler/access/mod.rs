mod admin;
mod app_base;
mod app_oauth;
mod app_sender;
mod app_sender_mail;
mod app_sender_sms;
mod rbac;
mod relation;
mod setting;
mod user;
pub use admin::*;
pub use app_base::*;
pub use app_oauth::*;
pub use app_sender::*;
pub use app_sender_mail::*;
pub use app_sender_sms::*;
use lsys_rbac::{
    access_relation_tpl, access_res_tpl,
    dao::{RelationTpl, ResTpl},
};
pub use rbac::*;
pub use relation::*;
pub use setting::*;
pub use user::*;

pub fn relation_tpls() -> Vec<RelationTpl> {
    access_relation_tpl!(RelationApp)
}

pub fn res_tpls() -> Vec<ResTpl> {
    access_res_tpl!(
        AccessAdminSmtpConfig,
        AccessAppSenderMailConfig,
        AccessAppSenderMailMsg,
        AccessAppSenderDoMail,
        AccessAdminManage,
        AccessAdminChangeLogsView,
        AccessAdminDocsEdit,
        AccessAdminSetting,
        AccessAdminUserFull,
        AccessAdminUserBase,
        AccessAppSenderDoSms,
        AccessSubAppView,
        AccessSubAppRbacCheck,
        AccessOauthUserInfo,
        AccessOauthUserEmail,
        AccessOauthUserMobile,
        AccessOauthUserAddress,
        AccessAdminAliSmsConfig,
        AccessAppSenderSmsConfig,
        AccessAppSenderSmsMsg,
        AccessResView,
        AccessResEdit,
        AccessRoleView,
        AccessRoleEdit,
        AccessRoleViewList,
        AccessUserAppConfirm,
        AccessUserMobileEdit,
        AccessUserMobileView,
        AccessUserAppEdit,
        AccessUserAppView,
        AccessUserEmailEdit,
        AccessUserEmailView,
        AccessUserInfoEdit,
        AccessUserNameEdit,
        AccessUserAddressEdit,
        AccessUserAddressView,
        AccessSystemLogin,
        AccessSystemEmailConfirm,
        AccessSystemMobileConfirm,
        AccessSystemReSetPassword,
        AccessUserExternalEdit,
        AccessUserSetPassword,
        AccessAdminSenderTplView,
        AccessAdminSenderTplEdit,
        AccessSiteSetting
    )
}
