use crate::configuration::{ConfigEnv, DatabaseSettings, GlobalSettings};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use super::{check_db_exists, init_and_migrate_db};

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
        // options.log_statements(tracing_log::log::LevelFilter::Trace);
    }
}

impl DbPool {
    #[tracing::instrument(name = "Initialize DbPool from Database Environment")]
    pub async fn init_pool(env: ConfigEnv) -> anyhow::Result<DbPool> {
        let settings = env.get_settings().expect("failed to get settings").database;
        let pool = DbPool(
            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_millis(2000))
                .connect_with(settings.without_db())
                .await
                .expect("Failed to init pool from PoolOptions"),
        );
        if !check_db_exists(&pool, &settings.database_name).await {
            tracing::error!(
                "Database needs to be initialized and migrated!\nHave you run scripts/init_db.sh?"
            )
        }
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
