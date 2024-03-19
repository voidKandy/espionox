#[cfg(feature = "bert")]
pub mod huggingface;

pub mod anthropic;

pub mod completion_handler;
pub mod error;
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
    completion_handler::{EndpointCompletionHandler, LLMCompletionHandler},
    error::ModelEndpointError,
    openai::{
        completions::{streaming::CompletionStream, OpenAiCompletionHandler},
        functions::Function,
    },
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ModelProvider {
    OpenAi,
    Anthropic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLM {
    handler: LLMCompletionHandler,
    /// Temperature is computed to a number between 0 and 1 by dividing this value by 100
    temperature: i32,
    token_count: i32,
}

impl LLM {
    pub fn provider(&self) -> ModelProvider {
        match self.handler {
            LLMCompletionHandler::OpenAi(_) => ModelProvider::OpenAi,
            LLMCompletionHandler::Anthropic(_) => ModelProvider::Anthropic,
        }
    }

    pub fn new(handler: LLMCompletionHandler, temperature: i32) -> LLM {
        Self {
            handler,
            temperature,
            token_count: 0,
        }
    }

    fn temperature(&self) -> f32 {
        (self.temperature / 100) as f32
    }
    /// Default openai handler with 0.7 temp
    pub fn default_openai() -> LLM {
        let handler = OpenAiCompletionHandler::default().into();
        LLM::new(handler, 70)
    }

    /// Default anthropic handler with 0.7 temp
    pub fn default_anthropic() -> LLM {
        let handler = AnthropicCompletionHandler::default().into();
        LLM::new(handler, 70)
    }

    pub async fn get_io_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<String, ModelEndpointError> {
        let comp_handler: Box<dyn EndpointCompletionHandler> = self.handler.comp_handler();
        let headers = comp_handler.request_headers(api_key);
        let body = comp_handler.io_request_body(messages, self.temperature());
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
        let comp_handler: Box<dyn EndpointCompletionHandler> = self.handler.comp_handler();
        let headers = comp_handler.request_headers(api_key);
        let body = comp_handler.stream_request_body(messages, self.temperature())?;
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
        let comp_handler: Box<dyn EndpointCompletionHandler> = self.handler.comp_handler();
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
}
