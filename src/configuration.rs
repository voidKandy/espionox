#[cfg(feature = "long_term_memory")]
use serde_yaml;
use std::{collections::BTreeMap, fs, io::Read, path::PathBuf};

use crate::language_models::openai::gpt::GptModel;

#[derive(Debug, PartialEq, Clone)]
pub struct ConfigEnv {
    config_file_name: String,
}

impl Default for ConfigEnv {
    fn default() -> Self {
        Self {
            config_file_name: "default".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct GlobalSettings {
    pub language_model: LanguageModelSettings,
    pub database: Option<DatabaseSettings>,
}

struct GlobalSettingsBuilder {
    lm: Option<LanguageModelSettings>,
    db: Option<DatabaseSettings>,
}

#[derive(Clone, Debug)]
pub struct LanguageModelSettings {
    pub default_model: GptModel,
    pub api_key: String,
}

#[derive(Clone, Debug)]
pub struct DatabaseSettings {
    pub port: u16,
    pub username: String,
    pub password: String,
    pub host: String,
    pub database_name: String,
}

type SettingsYamlMap = BTreeMap<String, String>;
type GlobalSettingsYamlMap = BTreeMap<String, SettingsYamlMap>;

fn global_yaml_map_from_path(path: PathBuf) -> GlobalSettingsYamlMap {
    let mut file = fs::File::open(path).expect("Failed to read default config path");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Failed to read file");
    let yaml_map: GlobalSettingsYamlMap = serde_yaml::from_str(&content).unwrap();
    yaml_map
}

trait OverwriteWith<T> {
    fn overwrite_with(&mut self, _: T) {}
}

impl OverwriteWith<SettingsYamlMap> for LanguageModelSettings {
    fn overwrite_with(&mut self, map: SettingsYamlMap) {
        for (k, val) in map {
            match k.as_str() {
                "model" => {
                    self.default_model = val.try_into().unwrap();
                }
                "api_key" => {
                    self.api_key = val;
                }
                _ => {}
            }
        }
    }
}

#[cfg(feature = "long_term_memory")]
impl OverwriteWith<SettingsYamlMap> for DatabaseSettings {
    fn overwrite_with(&mut self, map: SettingsYamlMap) {
        for (key, val) in map {
            let val = val.to_string();
            match key.as_str() {
                "port" => {
                    self.port = val.parse().unwrap();
                }
                "username" => {
                    self.username = val;
                }
                "password" => {
                    self.password = val;
                }
                "host" => {
                    self.host = val;
                }
                "database_name" => {
                    self.database_name = val;
                }
                _ => {}
            }
        }
    }
}

impl OverwriteWith<GlobalSettingsYamlMap> for GlobalSettings {
    fn overwrite_with(&mut self, global_map: GlobalSettingsYamlMap) {
        for (key, map) in global_map {
            match key.as_str() {
                "language_model" => self.language_model.overwrite_with(map),

                #[cfg(feature = "long_term_memory")]
                "database" => {
                    if let Some(database) = self.database.as_mut() {
                        database.overwrite_with(map);
                    }
                }
                _ => {}
            }
        }
    }
}

impl TryFrom<&SettingsYamlMap> for LanguageModelSettings {
    type Error = anyhow::Error;
    fn try_from(map: &SettingsYamlMap) -> Result<Self, Self::Error> {
        let mut model: Option<GptModel> = None;
        let mut api_key: Option<String> = None;
        for (key, val) in map {
            let val = val.to_string();
            match key.as_str() {
                "model" => {
                    model = Some(val.try_into().unwrap());
                }
                "api_key" => {
                    api_key = Some(val);
                }
                _ => {}
            }
        }
        let default_model = match model {
            None => {
                tracing::info!("No model in language model settings, using default");
                GptModel::default()
            }
            Some(m) => m,
        };
        Ok(LanguageModelSettings {
            default_model,
            api_key: api_key.ok_or_else(|| anyhow::anyhow!("Missing api key"))?,
        })
    }
}

impl TryFrom<&SettingsYamlMap> for DatabaseSettings {
    type Error = anyhow::Error;
    fn try_from(map: &SettingsYamlMap) -> Result<Self, Self::Error> {
        let mut port: Option<u16> = None;
        let mut username: Option<String> = None;
        let mut password: Option<String> = None;
        let mut host: Option<String> = None;
        let mut database_name: Option<String> = None;
        for (key, val) in map {
            let val = val.to_string();
            match key.as_str() {
                "port" => {
                    port = Some(val.parse().unwrap());
                }
                "username" => {
                    username = Some(val);
                }
                "password" => {
                    password = Some(val);
                }
                "host" => {
                    host = Some(val);
                }
                "database_name" => {
                    database_name = Some(val);
                }
                _ => {}
            }
        }
        Ok(DatabaseSettings {
            port: port.ok_or_else(|| anyhow::anyhow!("Port is missing"))?,
            username: username.ok_or_else(|| anyhow::anyhow!("Username is missing"))?,
            password: password.ok_or_else(|| anyhow::anyhow!("Password is missing"))?,
            host: host.ok_or_else(|| anyhow::anyhow!("Host is missing"))?,
            database_name: database_name
                .ok_or_else(|| anyhow::anyhow!("Database name is missing"))?,
        })
    }
}

impl TryFrom<GlobalSettingsYamlMap> for GlobalSettingsBuilder {
    type Error = anyhow::Error;
    fn try_from(yaml_map: GlobalSettingsYamlMap) -> Result<Self, Self::Error> {
        let mut global_settings_builder = GlobalSettings::build();
        for (setting, map) in yaml_map.iter() {
            match setting.as_str() {
                "database" => {
                    global_settings_builder.db = Some(DatabaseSettings::try_from(map).unwrap());
                }
                "language_model" => {
                    global_settings_builder.lm =
                        Some(LanguageModelSettings::try_from(map).unwrap());
                }
                _ => {}
            }
        }
        Ok(global_settings_builder)
    }
}

impl TryInto<GlobalSettings> for GlobalSettingsBuilder {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<GlobalSettings, Self::Error> {
        Ok(GlobalSettings {
            language_model: self
                .lm
                .ok_or_else(|| anyhow::anyhow!("Language model settings missing"))?,
            database: self.db,
        })
    }
}

impl GlobalSettings {
    fn build() -> GlobalSettingsBuilder {
        GlobalSettingsBuilder { lm: None, db: None }
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
        let configuration_dir = base_path.join("espionox_config");
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
    pub fn global_settings(&self) -> Result<GlobalSettings, anyhow::Error> {
        let default_config_path = Self::default().config_file_path();
        let default_config_override: Option<PathBuf> = match self.config_file_name.as_str() {
            "default" => {
                tracing::info!("Using default path");
                None
            }
            path => {
                tracing::info!("Using override path: {}", path);
                Some(self.config_file_path())
            }
        };

        let df_map = global_yaml_map_from_path(default_config_path);
        let mut df_settings: GlobalSettings = GlobalSettingsBuilder::try_from(df_map)
            .unwrap()
            .try_into()
            .unwrap();
        if let Some(path) = default_config_override {
            let override_map = global_yaml_map_from_path(path);
            df_settings.overwrite_with(override_map);
        }

        tracing::info!("Got default global settings: {:?}", df_settings);
        Ok(df_settings)
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
