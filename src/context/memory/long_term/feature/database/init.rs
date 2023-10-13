use crate::configuration::{ConfigEnv, DatabaseSettings};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

#[derive(Clone, Debug)]
pub struct DbPool(sqlx::PgPool);

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

impl Default for DbPool {
    fn default() -> Self {
        DbPool::sync_init_pool(ConfigEnv::default())
    }
}

impl DbPool {
    #[tracing::instrument(name = "Initialize DbPool from Database Environment")]
    pub async fn init_pool(env: ConfigEnv) -> anyhow::Result<DbPool> {
        let settings = env
            .global_settings()
            .expect("failed to get settings")
            .database
            .expect("No database settings");
        tracing::info!("Connecting to {:?}", settings.without_db());
        let pool = DbPool(
            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_millis(2000))
                .connect_with(settings.with_db())
                .await
                .expect("Failed to init pool from PoolOptions"),
        );
        Ok(pool)
    }

    #[tracing::instrument(name = "Synchronously initialize DbPool from Database Environment")]
    pub fn sync_init_pool(env: ConfigEnv) -> DbPool {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                DbPool::init_pool(env)
                    .await
                    .expect("Failed to initialize pool")
            })
        })
        .join()
        .expect("Failed to init long term DbPool")
    }
}

impl AsRef<sqlx::PgPool> for DbPool {
    fn as_ref(&self) -> &sqlx::PgPool {
        &self.0
    }
}
