use crate::configuration::ConfigEnv;

use super::functions::config::Function;
use anyhow::anyhow;
use bytes::Bytes;
use futures::Stream;
#[allow(unused)]
use futures_util::StreamExt;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::error::Error;

pub use super::errors::GptError;

// Add tokens and status fields here
#[derive(Debug, Deserialize, Clone)]
pub struct GptResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamResponse {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamChoice {
    pub delta: StreamDelta,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub message: GptMessage,
}

pub type CompletionStream = Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin>;

#[derive(Debug, Deserialize, Clone)]
pub struct GptMessage {
    pub role: String,
    pub content: Option<String>,
    pub function_call: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct Gpt {
    pub config: GptConfig,
}

#[derive(Clone, Debug, Default)]
pub struct GptConfig {
    api_key: String,
    client: Client,
    url: String,
    model: String,
}

impl GptResponse {
    pub fn parse(&self) -> Result<String, Box<dyn Error>> {
        // println!("{:?}", &self);
        match self.choices[0].message.content.to_owned() {
            Some(response) => Ok(response),
            None => Err("Unable to parse completion response".into()),
        }
    }

    #[tracing::instrument]
    pub fn parse_fn(&self) -> Result<Value, Box<dyn Error>> {
        match self
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
            None => Err("Unable to parse completion response".into()),
        }
    }
}

impl StreamResponse {
    #[tracing::instrument(name = "Get token from byte chunk")]
    pub async fn from_byte_chunk(chunk: Bytes) -> Result<Option<Self>, GptError> {
        let chunk_string = String::from_utf8_lossy(&chunk).trim().to_string();

        let chunk_strings: Vec<&str> = chunk_string.split('\n').filter(|s| !s.is_empty()).collect();

        tracing::info!(
            "{} chunk data strings to process: {:?}",
            chunk_strings.len(),
            chunk_strings
        );
        for string in chunk_strings
            .iter()
            .map(|s| s.trim_start_matches("data:").trim())
        {
            tracing::info!("Processing string: {}", string);
            if string == "[DONE]" {
                return Ok(None);
            }

            match serde_json::from_str::<StreamResponse>(&string) {
                Ok(stream_response) => {
                    if let Some(choice) = &stream_response.choices.get(0) {
                        tracing::info!("Chunk as stream response: {:?}", stream_response);
                        if choice.delta.role.is_some() {
                            continue;
                        }
                        if choice.delta.content.is_none() && choice.delta.role.is_none() {
                            continue;
                        }
                        return Ok(Some(stream_response));
                    }
                }
                Err(err) => {
                    if err.to_string().contains("expected value") {
                        return Err(GptError::Recoverable(format!(
                            "Possibly recoverable error: {:?}",
                            err
                        )));
                    }
                }
            }
        }
        Ok(None)
    }

    #[tracing::instrument(name = "Parse stream response for string")]
    pub fn parse(&self) -> Result<String, Box<dyn Error>> {
        tracing::info!(
            "self.choices[0].delta.content: {}",
            self.choices[0]
                .delta
                .content
                .to_owned()
                .expect("Failed to get delta content")
        );
        match self.choices[0].delta.content.to_owned() {
            Some(response) => Ok(response
                .trim_start_matches('"')
                .trim_end_matches('"')
                .to_string()),
            None => Err("Unable to parse stream completion response".into()),
        }
    }
}

impl GptConfig {
    pub fn init(env: ConfigEnv) -> GptConfig {
        let settings = env
            .get_settings()
            .expect("Failed to get model settings")
            .language_model;
        let api_key = settings.api_key;
        let model = settings.model;
        let client = Client::new();
        let url = "https://api.openai.com/v1/chat/completions".to_string();
        GptConfig {
            api_key,
            client,
            url,
            model,
        }
    }
}

impl Default for Gpt {
    fn default() -> Self {
        let config = GptConfig::init(ConfigEnv::default());
        Gpt { config }
    }
}

impl Gpt {
    pub fn handle_completion_error(err: Box<dyn Error>) -> GptResponse {
        if err.to_string().contains("missing field `choices`") {
            let message = format!("Something trivial went wrong please try again");
            GptResponse {
                choices: vec![Choice {
                    message: GptMessage {
                        role: "system".to_string(),
                        content: Some(message),
                        function_call: None,
                    },
                }],
            }
        } else {
            panic!("An unexpected error occurred: {}", err)
        }
    }

    #[tracing::instrument(name = "Get stream completion")]
    pub async fn stream_completion(
        &self,
        context: &Vec<Value>,
    ) -> Result<CompletionStream, GptError> {
        let payload = json!({
            "model": self.config.model,
            "messages": context,
            "stream": true,
            "max_tokens": 1000,
            "n": 1,
            "stop": null,
        });
        tracing::info!("PAYLOAD: {:?}", &payload);

        let response_stream = self
            .config
            .client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|err| GptError::Completion(err))?
            .bytes_stream();

        Ok(Box::new(response_stream))
    }

    pub async fn completion(&self, context: &Vec<Value>) -> Result<GptResponse, GptError> {
        let payload = json!({"model": self.config.model, "messages": context, "max_tokens": 1000, "n": 1, "stop": null});
        tracing::info!("PAYLOAD: {:?}", &payload);
        match self
            .config
            .client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => response.json().await.map_err(|err| {
                GptError::Undefined(anyhow!("Error getting response Json: {err:?}"))
            }),
            Err(err) => {
                println!("Completion Error: {err:?}");
                Err(GptError::Completion(err))
            }
        }
    }

    #[tracing::instrument(name = "Get function completion" skip(context, function))]
    pub async fn function_completion(
        &self,
        context: &Vec<Value>,
        function: &Function,
    ) -> Result<GptResponse, GptError> {
        let payload = json!({
            "model": self.config.model,
            "messages": context,
            "functions": [function.json],
            "function_call": {"name": function.name}
        });
        tracing::info!("Full completion payload: {:?}", payload);
        let response = self
            .config
            .client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        let gpt_response = response.json().await?;
        Ok(gpt_response)
    }
}
