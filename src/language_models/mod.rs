#[cfg(feature = "bert")]
pub mod huggingface;

pub mod anthropic;

pub mod error;
pub mod inference;
pub mod openai;

use anyhow::anyhow;
use reqwest_streams::JsonStreamResponse;

use crate::{
    environment::agent_handle::MessageStack,
    language_models::openai::completions::streaming::StreamResponse,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Debug, time::Duration};

use self::{
    anthropic::AnthropicCompletionHandler,
    error::ModelEndpointError,
    inference::{
        CompletionEndpointHandler, EmbeddingEndpointHandler, LLMCompletionHandler,
        LLMEmbeddingHandler, LLMInferenceHandler,
    },
    openai::completions::{
        functions::Function, streaming::CompletionStream, OpenAiCompletionHandler,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelProvider {
    OpenAi,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLM {
    handler: LLMInferenceHandler,
    /// Temperature is computed to a number between 0 and 1 by dividing this value by 100
    params: ModelParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// Total token usage count of the model
    pub total_token_count: u32,
    /// What sampling temperature to use, between 0 and 2.
    /// Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic.
    /// Input as a value between 0 and 200. Will be turned into float.
    pub temperature: Option<u8>,
    /// Number between -2.0 and 2.0.
    /// Positive values penalize new tokens based on their existing frequency in the text so far,
    /// decreasing the model's likelihood to repeat the same line verbatim.
    pub frequency_penalty: Option<i8>,
    /// The maximum number of tokens that can be generated in the chat completion.
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    pub max_tokens: Option<u32>,
    /// How many chat completion choices to generate for each input message.
    /// Note that you will be charged based on the number of generated tokens across all of the choices.
    /// Keep n as 1 to minimize costs.
    pub n: Option<u32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<i8>,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            total_token_count: 0,
            temperature: Some(70),
            frequency_penalty: None,
            max_tokens: None,
            n: Some(1),
            presence_penalty: None,
        }
    }
}

impl ModelParameters {
    fn temperature(&self) -> Result<f32, anyhow::Error> {
        Ok((self.temperature.ok_or(anyhow!("No temperature"))? / 100) as f32)
    }
}

impl LLM {
    pub fn provider(&self) -> ModelProvider {
        self.handler.provider()
    }

    pub fn new_completion_model(
        handler: LLMCompletionHandler,
        params: Option<ModelParameters>,
    ) -> LLM {
        let params = match params {
            None => ModelParameters::default(),
            Some(p) => p,
        };
        let handler = handler.into();
        Self { handler, params }
    }

    pub fn new_embedding_model(
        handler: LLMEmbeddingHandler,
        params: Option<ModelParameters>,
    ) -> LLM {
        let params = match params {
            None => ModelParameters::default(),
            Some(p) => p,
        };
        let handler = handler.into();
        Self { handler, params }
    }

    /// Default openai handler with 0.7 temp
    pub fn default_openai() -> LLM {
        let handler = OpenAiCompletionHandler::default().into();
        LLM::new_completion_model(handler, None)
    }

    /// Default anthropic handler with 0.7 temp
    pub fn default_anthropic() -> LLM {
        let handler = AnthropicCompletionHandler::default().into();
        LLM::new_completion_model(handler, None)
    }

    pub async fn get_io_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<String, ModelEndpointError> {
        let comp_handler: Box<dyn CompletionEndpointHandler> = self.handler.comp_handler()?;
        let headers = comp_handler.request_headers(api_key);
        let body = comp_handler.io_request_body(messages, self.params.temperature()?);
        let response = client
            .post(comp_handler.completion_url())
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let json = response.json().await?;
        Ok(comp_handler.handle_io_response(json)?)
    }

    pub async fn get_stream_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<CompletionStream, ModelEndpointError> {
        let comp_handler: Box<dyn CompletionEndpointHandler> = self.handler.comp_handler()?;
        let headers = comp_handler.request_headers(api_key);
        let body = comp_handler.stream_request_body(messages, self.params.temperature()?)?;
        let request = client
            .post(comp_handler.completion_url())
            .headers(headers)
            .json(&body);
        let response_stream = tokio::time::timeout(Duration::from_secs(10), async {
            request
                .send()
                .await
                .map_err(|err| ModelEndpointError::NetRequest(err))
                .unwrap()
                .json_array_stream::<StreamResponse>(1024)
        })
        .await
        .map_err(|_| ModelEndpointError::Undefined(anyhow!("Response stream request timed out")))?;
        tracing::info!("Got response stream, returning");
        Ok(Box::new(response_stream))
    }

    pub async fn get_fn_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
        function: Function,
    ) -> Result<Value, ModelEndpointError> {
        let comp_handler: Box<dyn CompletionEndpointHandler> = self.handler.comp_handler()?;
        let headers = comp_handler.request_headers(api_key);
        let body = comp_handler.fn_request_body(messages, function)?;
        let response = client
            .post(comp_handler.completion_url())
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let json = response.json().await?;
        Ok(comp_handler.handle_fn_response(json)?)
    }

    pub async fn get_embedding(
        &self,
        text: &str,
        api_key: &str,
        client: &Client,
    ) -> Result<Vec<f32>, ModelEndpointError> {
        let emb_handler: Box<dyn EmbeddingEndpointHandler> = self.handler.emb_handler()?;
        let headers = emb_handler.request_headers(api_key);
        let body = emb_handler.request_body(text);
        let response = client
            .post(emb_handler.completion_url())
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let json = response.json().await?;
        Ok(emb_handler.handle_response(json)?)
    }
}
