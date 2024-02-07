//统一错误

use deadpool_redis::PoolError;

use crate::dao::account::UserAccountError;
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidCodeError};

use redis::RedisError;

// use std::error::Error;
// use std::fmt::{Display, Formatter};

use std::string::FromUtf8Error;

use std::time::SystemTimeError;

#[derive(Debug)]
pub enum UserAuthError {
    TokenParse(FluentMessage),
    Sqlx(sqlx::Error),
    Redis(RedisError),
    RedisPool(PoolError),
    PasswordNotMatch((u64, FluentMessage)),
    PasswordNotSet((u64, FluentMessage)),
    StatusError((u64, FluentMessage)),
    ValidCode(ValidCodeError),
    UserNotFind(FluentMessage),
    NotLogin(FluentMessage),
    UserAccount(UserAccountError),
    System(FluentMessage),
    SerdeJson(serde_json::Error),
    CheckUserLock((u64, FluentMessage)),
    CheckCaptchaNeed(FluentMessage),
    Utf8Err(FromUtf8Error),
}

impl IntoFluentMessage for UserAuthError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            UserAuthError::TokenParse(err) => err.to_owned(),
            UserAuthError::Sqlx(err) => fluent_message!("sqlx-error", err),
            UserAuthError::Redis(err) => fluent_message!("redis-error", err),
            UserAuthError::RedisPool(err) => fluent_message!("redis-error", err),
            UserAuthError::PasswordNotMatch(err) => err.1.to_owned(),
            UserAuthError::PasswordNotSet(err) => err.1.to_owned(),
            UserAuthError::StatusError(err) => err.1.to_owned(),
            UserAuthError::ValidCode(err) => err.to_fluent_message(),
            UserAuthError::UserNotFind(err) => err.to_owned(),
            UserAuthError::NotLogin(err) => err.to_owned(),
            UserAuthError::UserAccount(err) => err.to_fluent_message(),
            UserAuthError::System(err) => err.to_owned(),
            UserAuthError::CheckUserLock(err) => err.1.to_owned(),
            UserAuthError::CheckCaptchaNeed(err) => err.to_owned(),
            UserAuthError::SerdeJson(err) => fluent_message!("serde-json-error", err),
            UserAuthError::Utf8Err(err) => fluent_message!("utf-parse-error", err),
        }
    }
}

// impl Display for UserAuthError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
// impl Error for UserAuthError {}

pub type UserAuthResult<T> = Result<T, UserAuthError>;

impl From<sqlx::Error> for UserAuthError {
    fn from(err: sqlx::Error) -> Self {
        UserAuthError::Sqlx(err)
    }
}
impl From<SystemTimeError> for UserAuthError {
    fn from(err: SystemTimeError) -> Self {
        UserAuthError::System(fluent_message!("time-error", err))
    }
}
impl From<RedisError> for UserAuthError {
    fn from(err: RedisError) -> Self {
        UserAuthError::Redis(err)
    }
}
impl From<PoolError> for UserAuthError {
    fn from(err: PoolError) -> Self {
        UserAuthError::RedisPool(err)
    }
}
impl From<serde_json::Error> for UserAuthError {
    fn from(err: serde_json::Error) -> Self {
        UserAuthError::SerdeJson(err)
    }
}
impl From<UserAccountError> for UserAuthError {
    fn from(err: UserAccountError) -> Self {
        UserAuthError::UserAccount(err)
    }
}
impl From<FromUtf8Error> for UserAuthError {
    fn from(err: FromUtf8Error) -> Self {
        UserAuthError::Utf8Err(err)
    }
}
impl From<ValidCodeError> for UserAuthError {
    fn from(err: ValidCodeError) -> Self {
        UserAuthError::ValidCode(err)
    }
}
