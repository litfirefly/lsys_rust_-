use std::sync::Arc;

pub mod rbac;

pub use self::rbac::*;

use self::rbac::Rbac;
use lsys_core::{AppCore, AppCoreError, FluentMessage};
use redis::aio::ConnectionManager;
use sqlx::{MySql, Pool};
use tokio::sync::Mutex;

pub struct RbacDao {
    //内部依赖
    pub fluent: Arc<FluentMessage>,
    pub db: Pool<MySql>,
    pub redis: Arc<Mutex<ConnectionManager>>,
    // 权限相关
    pub rbac: Arc<Rbac>,
}

impl RbacDao {
    pub async fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        redis: Arc<Mutex<ConnectionManager>>,
        system_role: Option<Box<dyn SystemRoleCheckData>>,
        use_cache: bool,
    ) -> Result<RbacDao, AppCoreError> {
        let app_locale_dir = app_core.app_dir.join("locale/lsys-rbac");
        let fluents_message = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });
        let rbac = Arc::from(Rbac::new(
            fluents_message.clone(),
            db.clone(),
            redis.clone(),
            system_role,
            use_cache,
        ));
        Ok(RbacDao {
            fluent: fluents_message,
            rbac,
            db,
            redis,
        })
    }
}
