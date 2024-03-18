use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::environment::agent_handle::{Message, MessageRole, MessageStack};

use super::{endpoint_completions::EndpointCompletionHandler, error::ModelEndpointError};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum AnthropicCompletionHandler {
    #[default]
    Opus,
    Sonnet,
    Haiku,
}

const OPUS_MODEL_STR: &str = "claude-3-opus-20240229";
const SONNET_MODEL_STR: &str = "claude-3-sonnet-20240229";
const HAIKU_MODEL_STR: &str = "claude-3-haiku-20240307";

impl EndpointCompletionHandler for AnthropicCompletionHandler {
    fn provider(&self) -> super::ModelProvider {
        super::ModelProvider::Anthropic
    }
    fn name(&self) -> &str {
        match self {
            Self::Opus => OPUS_MODEL_STR,
            Self::Sonnet => SONNET_MODEL_STR,
            Self::Haiku => HAIKU_MODEL_STR,
        }
    }

    fn context_window(&self) -> i64 {
        200000
    }

    fn from_str(str: &str) -> Option<Self> {
        match str {
            OPUS_MODEL_STR => Some(Self::Opus),
            SONNET_MODEL_STR => Some(Self::Sonnet),
            HAIKU_MODEL_STR => Some(Self::Haiku),
            _ => None,
        }
    }
    fn completion_url(&self) -> &str {
        "https://api.anthropic.com/v1/messages"
    }

    fn agent_cache_to_json(cache: &MessageStack) -> Vec<Value> {
        // Anthropic model requires that messages alternate from User to assistant. So we'll
        // concatenate all adjacent messages to one
        let mut val_vec: Vec<Value> = vec![];
        let mut last_message: Option<Message> = None;

        for message in cache.clone().into_iter() {
            match last_message.take() {
                Some(mut m) => {
                    if message.role == m.role {
                        m.content = format!("{}. {}", m.content, message.content);
                        last_message = Some(m);
                    } else {
                        let val: Value = m.into();
                        val_vec.push(val);
                        last_message = Some(message);
                    }
                }
                None => last_message = Some(message),
            }
        }
        if let Some(m) = last_message {
            let val: Value = m.into();
            val_vec.push(val);
        }
        val_vec
    }

    fn request_headers(&self, api_key: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert("x-api-key", format!("{}", api_key).parse().unwrap());
        map.insert("anthropic-version", "2023-06-01".parse().unwrap());
        map.insert("content-type", "application/json".parse().unwrap());
        map
    }

    fn io_request_body(&self, messages: &MessageStack, temperature: f32) -> Value {
        let system_stack: MessageStack = messages.ref_filter_by(MessageRole::System, true).into();
        let sans_system_stack: MessageStack =
            messages.ref_filter_by(MessageRole::System, false).into();
        let context = Self::agent_cache_to_json(&sans_system_stack);
        let system_content = system_stack
            .as_ref()
            .into_iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<&str>>();
        match system_content.is_empty() {
            true => {
                json!({"model": self.name(), "messages": context, "temperature": temperature, "max_tokens": 1024})
            }
            false => {
                json!({"model": self.name(), "messages": context, "temperature": temperature, "max_tokens": 1024, "system": system_content.join(".")})
            }
        }
    }

    fn handle_io_response(&self, response: Value) -> Result<String, ModelEndpointError> {
        // println!("value: {:?}", response);
        let response = AnthropicResponse::try_from(response).unwrap();
        Ok(response.content[0].text.to_owned())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicResponse {
    content: Vec<AnthropicResponseContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicResponseContent {
    text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}

impl TryFrom<Value> for AnthropicResponse {
    type Error = anyhow::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let response: AnthropicResponse = serde_json::from_value(value)?;
        Ok(response)
    }
}
