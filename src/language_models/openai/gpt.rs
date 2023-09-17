use crate::configuration::ConfigEnv;

use super::functions::config::Function;
use bytes::Bytes;
use futures::Stream;
#[allow(unused)]
use futures_util::StreamExt;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::error::Error;
use tracing::info;

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
    pub delta: GptMessage,
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
    pub async fn from_byte_chunk(
        chunk: Bytes,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let start_chunk_pattern = "data: {";
        let end_data_pattern = "data: [DONE]";

        let chunk_str = String::from_utf8_lossy(&chunk).trim().to_string();
        let chunk_idcs: Vec<usize> = chunk_str
            .match_indices(start_chunk_pattern)
            .map(|(idx, _)| idx)
            .collect();

        info!(
            "Chunk String: {}\n, Number of true chunks: {}\nIndices: {:?}",
            &chunk_str,
            chunk_idcs.len(),
            chunk_idcs
        );

        for (i, idx) in chunk_idcs.iter().enumerate() {
            let mut slice = match i < chunk_idcs.len() - 1 {
                true => &chunk_str[idx + start_chunk_pattern.len() - 1..chunk_idcs[i + 1]],
                false => &chunk_str[idx + start_chunk_pattern.len() - 1..],
            };
            if let Some(done_idx) = slice.find(end_data_pattern) {
                slice = &slice[..done_idx];
            }
            let res = serde_json::from_str::<StreamResponse>(&slice)?;
            if let Some(string) = &res.choices[0].delta.content {
                if string.is_empty() {
                    continue;
                }
                return Ok(res);
            };
        }

        Err("No chunks processed, unexpected error. Likely no chunks we found.".into())
    }

    pub fn parse(&self) -> Result<String, Box<dyn Error>> {
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
        let config = GptConfig::init(ConfigEnv::Default);
        Gpt { config }
    }
}

impl Gpt {
    pub fn handle_completion_error(err: Box<dyn Error>) -> GptResponse {
        // Completions will randomly not return any choices, so we handle it
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
    ) -> Result<impl Stream<Item = Result<Bytes, reqwest::Error>>, Box<dyn Error>> {
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
            .await?
            .bytes_stream();

        Ok(response_stream)
    }

    pub async fn completion(&self, context: &Vec<Value>) -> Result<GptResponse, Box<dyn Error>> {
        let payload = json!({"model": self.config.model, "messages": context, "max_tokens": 1000, "n": 1, "stop": null});
        info!("PAYLOAD: {:?}", &payload);
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
            Ok(response) => {
                let gpt_response: GptResponse = response.json().await?;
                info!("GPT RESPONSE: {:?}", gpt_response);
                Ok(gpt_response)
            }
            Err(err) => {
                println!("Completion Error: {err:?}");
                Err(err.into())
            }
        }
    }

    #[tracing::instrument(name = "Get function completion" skip(context, function))]
    pub async fn function_completion(
        &self,
        context: &Vec<Value>,
        function: &Function,
    ) -> Result<GptResponse, Box<dyn Error>> {
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
        // Err("tst".into())
    }
}
