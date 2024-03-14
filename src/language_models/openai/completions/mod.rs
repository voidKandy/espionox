pub mod streaming;

use crate::{
    environment::agent_handle::MessageStack,
    language_models::{
        endpoint_completions::EndpointCompletionHandler, error::ModelEndpointError,
        openai::OpenAiUsage,
    },
};

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum OpenAiCompletionHandler {
    #[default]
    Gpt3,
    Gpt4,
}

const GPT3_MODEL_STR: &str = "gpt-3.5-turbo-0125";
const GPT4_MODEL_STR: &str = "gpt-4-0125-preview";

impl EndpointCompletionHandler for OpenAiCompletionHandler {
    fn provider(&self) -> crate::language_models::ModelProvider {
        crate::language_models::ModelProvider::OpenAi
    }
    fn from_str(str: &str) -> Option<Self> {
        match str {
            GPT3_MODEL_STR => Some(Self::Gpt3),
            GPT4_MODEL_STR => Some(Self::Gpt4),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Gpt3 => GPT3_MODEL_STR,
            Self::Gpt4 => GPT4_MODEL_STR,
        }
    }

    fn context_window(&self) -> i64 {
        match self {
            Self::Gpt3 => 16385,
            Self::Gpt4 => 128000,
        }
    }

    fn completion_url(&self) -> &str {
        "https://api.openai.com/v1/chat/completions"
    }

    fn request_headers(&self, api_key: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        map.insert("Content-Type", "application/json".parse().unwrap());
        map
    }
    fn io_request_body(&self, messages: &MessageStack, temperature: f32) -> Value {
        let context: Vec<Value> = messages.into();
        json!({"model": self.name(), "messages": context, "temperature": temperature, "max_tokens": 1000, "n": 1, "stop": null})
    }
    fn fn_request_body(
        &self,
        messages: &MessageStack,
        function: super::functions::Function,
        temperature: f32,
    ) -> Result<Value, ModelEndpointError> {
        let context: Vec<Value> = messages.into();
        Ok(json!({
            "model": self.name(),
            "messages": context,
            "functions": [function.json],
            "function_call": {"name": function.name}
        }))
    }
    fn stream_request_body(
        &self,
        messages: &MessageStack,
        temperature: f32,
    ) -> Result<Value, ModelEndpointError> {
        let context: Vec<Value> = messages.into();
        Ok(json!({
            "model": self.name(),
            "messages": context,
            "temperature": temperature,
            "stream": true,
            "max_tokens": 1000,
            "n": 1,
            "stop": null,
        }))
    }
    fn handle_io_response(&self, response: Value) -> Result<String, ModelEndpointError> {
        let response = OpenAiResponse::try_from(response).unwrap();
        match response.choices[0].message.content.to_owned() {
            Some(response) => Ok(response),
            None => Err(ModelEndpointError::CouldNotParseResponse),
        }
    }
    fn handle_fn_response(&self, response: Value) -> Result<Value, ModelEndpointError> {
        let response = OpenAiResponse::try_from(response).unwrap();

        match response
            .choices
            .to_owned()
            .into_iter()
            .next()
            .unwrap()
            .message
            .function_call
        {
            Some(response) => {
                tracing::info!("Function response: {:?}", response);
                let args_json = serde_json::from_str::<Value>(
                    response
                        .get("arguments")
                        .expect("Couldn't parse arguments")
                        .as_str()
                        .unwrap(),
                )?;

                tracing::info!("Args json: {:?}", args_json);
                let mut args_output: Value = json!({});
                if let Some(arguments) = args_json.as_object() {
                    for (key, value) in arguments.iter() {
                        args_output
                            .as_object_mut()
                            .expect("Failed to get array mut")
                            .insert(key.to_string(), value.clone());
                    }
                }
                tracing::info!("Args output: {:?}", args_output);
                Ok(args_output)
            }
            None => Err(ModelEndpointError::CouldNotParseResponse),
        }
    }
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

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiResponse {
    pub choices: Vec<Choice>,
    pub usage: OpenAiUsage,
}

impl TryFrom<Value> for OpenAiResponse {
    type Error = anyhow::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let response: OpenAiResponse = serde_json::from_value(value)?;
        Ok(response)
    }
}
