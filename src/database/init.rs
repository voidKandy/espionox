use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use std::path::{Path, PathBuf};

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

#[derive(Debug, PartialEq)]
pub enum DatabaseEnv {
    Default,
    Testing,
}

impl DatabaseEnv {
    fn config_file_path(&self) -> PathBuf {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_dir = base_path.join("configuration");
        let filename = match self {
            DatabaseEnv::Default => "default",
            DatabaseEnv::Testing => "testing",
        };
        PathBuf::from(
            format!(
                "{}/{}.yaml",
                configuration_dir.display().to_string(),
                filename
            )
            .as_str(),
        )
    }

    #[tracing::instrument(name = "Mutate settings specific to env")]
    fn mutate_settings(&self, settings: &mut DatabaseSettings) {
        match self {
            DatabaseEnv::Default => {}
            DatabaseEnv::Testing => {
                let unique_id = uuid::Uuid::new_v4()
                    .to_string()
                    .split_once('-')
                    .expect("Uuid did not contain a '-'")
                    .0
                    .to_string();
                // settings.database_name = format!("{}_{}", settings.database_name, unique_id);
            }
        }
    }

    #[tracing::instrument(name = "Get settings from Database Environment")]
    pub fn get_settings(&self) -> Result<DatabaseSettings, config::ConfigError> {
        let default_config_path = DatabaseEnv::Default.config_file_path();
        let mut default_config_override: Option<PathBuf> = None;
        match self {
            DatabaseEnv::Default => {
                tracing::info!("Using default database configuration");
            }
            _ => {
                let filepath = self.config_file_path();
                tracing::info!(
                    "Using database configuration from {}",
                    filepath.display().to_string()
                );
                default_config_override = Some(filepath);
            }
        }
        let config = config::Config::builder().add_source(config::File::from(default_config_path));
        if let Some(path) = default_config_override {
            let config = config.add_source(config::File::from(path)).build()?;
            let mut settings = config
                .try_deserialize::<DatabaseSettings>()
                .expect("Failed to build custom settings");
            self.mutate_settings(&mut settings);
            return Ok(settings);
        }
        let config = config.build()?;
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
