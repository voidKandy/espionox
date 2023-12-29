use crate::configuration::ConfigEnv;

use super::error::GptError;
use anyhow::anyhow;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const GPT3_MODEL_STR: &str = "gpt-3.5-turbo-1106";
const GPT4_MODEL_STR: &str = "gpt-4";

// #[derive(Clone, Debug, Default, Serialize, Deserialize)]
// pub struct GptConfig {
//     pub(super) api_key: String,
//     #[serde(skip)]
//     pub(super) client: Client,
//     pub(super) url: String,
//     pub(super) model: GptModel,
// }

/// Gpt struct contains info needed for completion endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gpt {
    pub api_key: Option<String>,
    pub model: GptModel,
    pub token_count: i32,
    pub temperature: f32,
    #[serde(skip)]
    pub(crate) client: Client,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GptResponse {
    pub choices: Vec<Choice>,
    pub usage: GptUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub message: GptMessage,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct GptUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GptMessage {
    pub role: String,
    pub content: Option<String>,
    pub function_call: Option<Value>,
}

/// More variations of these models should be added
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum GptModel {
    #[default]
    Gpt3,
    Gpt4,
}

impl TryFrom<String> for GptModel {
    type Error = GptError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            GPT3_MODEL_STR => Ok(Self::Gpt3),
            GPT4_MODEL_STR => Ok(Self::Gpt4),
            _ => Err(GptError::Undefined(anyhow!(
                "{} does not have a corresponding GPT variant",
                value
            ))),
        }
    }
}

impl ToString for GptModel {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Gpt3 => GPT3_MODEL_STR,
            Self::Gpt4 => GPT4_MODEL_STR,
        })
    }
}

/// Sensible defaults for Gpt:
/// * Temperature of 0.7
/// * Gpt3 Model
/// * Api key from default ConfigEnv, or empty
impl Default for Gpt {
    fn default() -> Self {
        let env = ConfigEnv::default();
        let api_key = match env.global_settings() {
            Ok(settings) => settings.language_model.api_key,
            Err(_) => {
                tracing::warn!("API KEY not given, Completions will be unavailable");
                None
            }
        };
        let model = GptModel::default();
        let temperature = 0.7;
        let token_count = 0;
        let client = Client::new();
        Gpt {
            model,
            temperature,
            token_count,
            api_key,
            client,
        }
    }
}

impl Gpt {
    /// Create a GPT from a model, temperature, and api_key
    pub fn new(model: GptModel, temperature: f32, api_key: Option<&str>) -> Self {
        match api_key {
            None => {
                tracing::warn!("API KEY not given, if there is nothing in config environment completions will be unavailable");
                Self {
                    model,
                    temperature,
                    ..Default::default()
                }
            }
            Some(key) => Self {
                api_key: Some(key.to_string()),
                model,
                temperature,
                ..Default::default()
            },
        }
    }
}
