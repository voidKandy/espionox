use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::path::Path;

use super::{check_db_exists, init_and_migrate_db};

#[derive(Clone, Debug)]
pub struct DbPool(sqlx::PgPool);

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
    pub database_name: String,
}

#[derive(Debug)]
pub enum DatabaseEnv {
    Default,
    Testing,
}

impl DatabaseEnv {
    fn config_file_name(&self) -> String {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_dir = base_path.join("configuration");
        let filename = match self {
            DatabaseEnv::Default => "default",
            DatabaseEnv::Testing => "testing",
        };
        String::from(format!(
            "{}/{}.yaml",
            configuration_dir.display().to_string(),
            filename
        ))
    }

    #[tracing::instrument(name = "Get settings from Database Environment")]
    pub fn get_settings(&self) -> Result<DatabaseSettings, config::ConfigError> {
        let file = self.config_file_name();
        let filepath = Path::new(&file);
        tracing::info!(
            "Loading database configuration from {}",
            filepath.display().to_string()
        );
        let config = config::Config::builder()
            .add_source(config::File::from(filepath))
            .build()?;
        config.try_deserialize::<DatabaseSettings>()
    }
}

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
    pub async fn init_pool(env: DatabaseEnv) -> anyhow::Result<DbPool> {
        let settings = env.get_settings().expect("failed to get settings");
        let pool = DbPool(
            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_millis(2000))
                .connect_with(settings.without_db())
                .await
                .expect("Failed to init pool from PoolOptions"),
        );
        if !check_db_exists(&pool, &settings.database_name).await {
            tracing::info!("Database needs to be initialized and migrated...");
            init_and_migrate_db(&pool, settings).await.unwrap();
        }
        Ok(pool)
    }

    #[tracing::instrument(name = "Synchronously initialize DbPool from Database Environment")]
    pub fn sync_init_pool(env: DatabaseEnv) -> DbPool {
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
