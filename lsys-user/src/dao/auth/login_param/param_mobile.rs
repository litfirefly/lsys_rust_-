use crate::dao::account::UserAccount;
use crate::dao::account::UserAccountError;
use crate::dao::auth::{LoginParam, LoginType, UserAuthError, UserAuthResult};

use crate::model::{UserMobileModel, UserModel};
use async_trait::async_trait;
use lsys_core::get_message;
use lsys_core::FluentMessage;

use super::super::{LoginData, LoginEnv};
use super::auth_check_user_password;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::Select;
use std::sync::Arc;
use tokio::sync::Mutex;
pub struct MobileLogin {
    pub area_code: String,
    pub mobile: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobileLoginData(UserMobileModel);

impl MobileLoginData {
    pub async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        Ok(MobileLoginData(
            Select::type_new::<UserMobileModel>()
                .reload(&self.0, db)
                .await?,
        ))
    }
}

#[async_trait]
impl LoginParam for MobileLogin {
    async fn get_type(
        &self,
        _db: &Pool<MySql>,
        _redis: &Arc<Mutex<ConnectionManager>>,
        fluent: &Arc<FluentMessage>,
    ) -> UserAuthResult<LoginType> {
        Ok(LoginType {
            time_out: 3600 * 24,
            type_name: get_message!(fluent, "auth-login-type-mobile", "Mobile Login"),
        })
    }
    async fn get_user(
        &self,
        _db: &Pool<MySql>,
        _redis: &Arc<Mutex<ConnectionManager>>,
        fluent: &Arc<FluentMessage>,
        account: &Arc<UserAccount>,
        _: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        let mobile = account
            .user_mobile
            .find_by_last_mobile(self.area_code.clone(), self.mobile.clone())
            .await
            .map_err(auth_user_not_found_map!(fluent, self.show_name(), "mobile"))?;
        mobile.is_enable()?;
        let user =
            account
                .user
                .find_by_id(&mobile.user_id)
                .await
                .map_err(auth_user_not_found_map!(
                    fluent,
                    self.show_name(),
                    "mobile [user id]"
                ))?;
        user.is_enable()?;
        let user = auth_check_user_password(fluent, account, user, &self.password).await?;

        Ok((LoginData::Mobile(MobileLoginData(mobile)), user))
    }
    fn show_name(&self) -> String {
        format!("{}[{}]", self.mobile, self.area_code)
    }
}
