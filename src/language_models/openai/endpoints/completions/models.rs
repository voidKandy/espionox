use crate::language_models::{openai::endpoints::OpenAiUsage, ModelEndpointError};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const GPT3_MODEL_STR: &str = "gpt-3.5-turbo-0125";
const GPT4_MODEL_STR: &str = "gpt-4";

/// Gpt struct contains info needed for completion endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenAi {
    pub model: OpenAiCompletionModel,
    pub token_count: i32,
    pub temperature: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiResponse {
    pub choices: Vec<Choice>,
    pub usage: OpenAiUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub message: GptMessage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GptMessage {
    pub role: String,
    pub content: Option<String>,
    pub function_call: Option<Value>,
}

// More variations of these models should be added
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum OpenAiCompletionModel {
    #[default]
    Gpt3,
    Gpt4,
}

impl TryFrom<String> for OpenAiCompletionModel {
    type Error = ModelEndpointError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            GPT3_MODEL_STR => Ok(Self::Gpt3),
            GPT4_MODEL_STR => Ok(Self::Gpt4),
            _ => Err(ModelEndpointError::Undefined(anyhow!(
                "{} does not have a corresponding GPT variant",
                value
            ))),
        }
    }
}

impl ToString for OpenAiCompletionModel {
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
impl Default for OpenAi {
    fn default() -> Self {
        let model = OpenAiCompletionModel::default();
        let temperature = 0.7;
        let token_count = 0;
        OpenAi {
            model,
            temperature,
            token_count,
        }
    }
}

impl OpenAi {
    /// Create a GPT from a model, temperature, and api_key
    pub fn new(model: OpenAiCompletionModel, temperature: f32) -> Self {
        Self {
            model,
            temperature,
            ..Default::default()
        }
    }
}
