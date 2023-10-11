#[cfg(feature = "long_term_memory")]
use serde_aux::field_attributes::deserialize_number_from_string;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone)]
pub struct ConfigEnv {
    config_file_name: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct GlobalSettings {
    pub language_model: LanguageModelSettings,
    #[cfg(feature = "long_term_memory")]
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct LanguageModelSettings {
    pub model: String,
    pub api_key: String,
}

#[cfg(feature = "long_term_memory")]
#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
    pub database_name: String,
}

impl Default for ConfigEnv {
    fn default() -> Self {
        Self {
            config_file_name: "default".to_string(),
        }
    }
}

#[cfg(feature = "long_term_memory")]
impl std::fmt::Display for DatabaseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Database Url: postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

impl ConfigEnv {
    pub fn new(filename: &str) -> Self {
        Self {
            config_file_name: filename.to_string(),
        }
    }
    fn config_file_path(&self) -> PathBuf {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_dir = base_path.join("configuration");
        PathBuf::from(
            format!(
                "{}/{}.yaml",
                configuration_dir.display().to_string(),
                self.config_file_name
            )
            .as_str(),
        )
    }

    #[tracing::instrument(name = "Get settings from environment")]
    pub fn get_settings(&self) -> Result<GlobalSettings, config::ConfigError> {
        let default_config_path = Self::default().config_file_path();
        let mut default_config_override: Option<PathBuf> = None;
        match self.config_file_name.as_str() {
            "default" => {
                tracing::info!("Using default database configuration");
            }
            _ => {
                let filepath = self.config_file_path();
                tracing::info!(
                    "Using configuration from {}",
                    filepath.display().to_string()
                );
                default_config_override = Some(filepath);
            }
        }
        let config = config::Config::builder().add_source(config::File::from(default_config_path));
        if let Some(path) = default_config_override {
            let config = config.add_source(config::File::from(path)).build()?;
            let settings = config
                .try_deserialize::<GlobalSettings>()
                .expect("Failed to build custom settings");
            #[cfg(feature = "long_term_memory")]
            tracing::info!("Database url from settings: \n{}", settings.database);
            return Ok(settings);
        }
        let config = config.build()?;
        let settings = config
            .try_deserialize::<GlobalSettings>()
            .expect("Failed to build settings");
        #[cfg(feature = "long_term_memory")]
        tracing::info!("Database url from settings: \n{}", settings.database);
        Ok(settings)
    }
}
