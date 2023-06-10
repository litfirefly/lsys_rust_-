use serde::{Deserialize, Serialize};
use sqlx_model::SqlxModelStatus;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocGitStatus {
    Enable = 1,  //启用
    Delete = -1, //删除
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocGitTagStatus {
    Publish = 2, //已发布
    Build = 1,   //已添加
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocGitCloneStatus {
    Init = 1,    //待克隆
    Cloned = 2,  //已克隆
    Fail = 3,    //克隆失败
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocMenuStatus {
    Enable = 1,  //启用
    Delete = -1, //删除
}
