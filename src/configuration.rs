use serde_aux::field_attributes::deserialize_number_from_string;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum ConfigEnv {
    Default,
    Testing,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct GlobalSettings {
    pub language_model: LanguageModelSettings,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct LanguageModelSettings {
    pub model: String,
    pub api_key: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
    pub database_name: String,
}

impl ConfigEnv {
    fn config_file_path(&self) -> PathBuf {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_dir = base_path.join("configuration");
        let filename = match self {
            Self::Default => "default",
            Self::Testing => "testing",
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

    #[tracing::instrument(name = "Get settings from environment")]
    pub fn get_settings(&self) -> Result<GlobalSettings, config::ConfigError> {
        let default_config_path = Self::Default.config_file_path();
        let mut default_config_override: Option<PathBuf> = None;
        match self {
            Self::Default => {
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
                .try_deserialize::<GlobalSettings>()
                .expect("Failed to build custom settings");
            // self.mutate_settings(&mut settings);
            return Ok(settings);
        }
        let config = config.build()?;
        config.try_deserialize::<GlobalSettings>()
    }
}
