use crate::environment::errors::GptError;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const GPT3_MODEL_STR: &str = "gpt-3.5-turbo-1106";
const GPT4_MODEL_STR: &str = "gpt-4";

/// Gpt struct contains info needed for completion endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gpt {
    pub model: GptModel,
    pub token_count: i32,
    pub temperature: f32,
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
impl Default for Gpt {
    fn default() -> Self {
        let model = GptModel::default();
        let temperature = 0.7;
        let token_count = 0;
        Gpt {
            model,
            temperature,
            token_count,
        }
    }
}

impl Gpt {
    /// Create a GPT from a model, temperature, and api_key
    pub fn new(model: GptModel, temperature: f32) -> Self {
        Self {
            model,
            temperature,
            ..Default::default()
        }
    }
}
