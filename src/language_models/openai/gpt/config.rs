use super::error::GptError;
use anyhow::anyhow;
use bytes::Bytes;
use futures::Stream;
use reqwest::Client;
use reqwest_streams::error::StreamBodyError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GptConfig {
    pub(super) api_key: String,
    #[serde(skip)]
    pub(super) client: Client,
    pub(super) url: String,
    pub(super) model: GptModel,
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

/// Gpt struct contains info needed for completion endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gpt {
    pub config: GptConfig,
    pub token_count: i32,
    pub temperature: f32,
    pub model_override: Option<GptModel>,
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
            "gpt-3.5-turbo-1106" => Ok(Self::Gpt3),
            "gpt-4-1106" => Ok(Self::Gpt4),
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
            Self::Gpt3 => "gpt-3.5-turbo-1106",
            Self::Gpt4 => "gpt-4-1106",
        })
    }
}
