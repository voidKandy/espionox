use super::{anthropic::AnthropicCompletionHandler, error::*, ModelProvider};
use anyhow::anyhow;
use reqwest_streams::JsonStreamResponse;

use crate::{
    environment::agent_handle::MessageStack,
    language_models::openai::completions::streaming::StreamResponse,
};

use super::openai::{
    completions::{streaming::CompletionStream, OpenAiCompletionHandler},
    functions::Function,
};
use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Debug, time::Duration};

// pub trait EndpointCompletionModel: Sized + Clone + Copy {}

pub trait EndpointCompletionHandler: Clone + Copy + Debug + Sync + Send + 'static {
    fn provider(&self) -> ModelProvider;
    fn from_str(str: &str) -> Option<Self>;
    fn name(&self) -> &str;
    fn context_window(&self) -> i64;
    fn completion_url(&self) -> &str;
    fn request_headers(&self, api_key: &str) -> HeaderMap;
    fn io_request_body(&self, messages: &MessageStack, temperature: f32) -> Value;
    fn fn_request_body(
        &self,
        _messages: &MessageStack,
        _function: Function,
        _temperature: f32,
    ) -> Result<Value, ModelEndpointError> {
        Err(ModelEndpointError::MethodUnimplemented)
    }
    fn stream_request_body(
        &self,
        _messages: &MessageStack,
        _temperature: f32,
    ) -> Result<Value, ModelEndpointError> {
        Err(ModelEndpointError::MethodUnimplemented)
    }

    fn handle_io_response(&self, response: Value) -> Result<String, ModelEndpointError>;
    fn handle_fn_response(&self, _response: Value) -> Result<Value, ModelEndpointError> {
        Err(ModelEndpointError::MethodUnimplemented)
    }
    // fn handle_stream_response(&mut self, response: Value) -> Result<String, ModelEndpointError> {
    //     Err(ModelEndpointError::MethodUnimplemented)
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMCompletionHandler<H: EndpointCompletionHandler> {
    handler: H,
    /// Temperature is computed to a number between 0 and 1 by dividing this value by 100
    temperature: i32,
    token_count: i32,
}

impl<H: EndpointCompletionHandler> LLMCompletionHandler<H> {
    pub fn new(handler: H, temperature: i32) -> LLMCompletionHandler<H> {
        Self {
            handler,
            temperature,
            token_count: 0,
        }
    }

    fn temperature(&self) -> f32 {
        (self.temperature / 100) as f32
    }

    /// Returns immutable reference to inner Completion Handler
    pub fn inner_ref(&self) -> &H {
        &self.handler
    }
    /// Returns mutable reference to inner Completion Handler
    pub fn inner_mut(&mut self) -> &mut H {
        &mut self.handler
    }

    /// Default openai handler with 0.7 temp
    pub fn default_openai() -> LLMCompletionHandler<OpenAiCompletionHandler> {
        let handler = OpenAiCompletionHandler::default();
        LLMCompletionHandler::new(handler, 70)
    }

    /// Default anthropic handler with 0.7 temp
    pub fn default_anthropic() -> LLMCompletionHandler<AnthropicCompletionHandler> {
        let handler = AnthropicCompletionHandler::default();
        LLMCompletionHandler::new(handler, 70)
    }

    pub async fn get_io_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<String, ModelEndpointError> {
        let headers = self.inner_ref().request_headers(api_key);
        let body = self
            .inner_ref()
            .io_request_body(messages, self.temperature());
        let response = client
            .post(self.inner_ref().completion_url())
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let json = response.json().await?;
        Ok(self.inner_ref().handle_io_response(json)?)
    }

    pub async fn get_stream_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<CompletionStream, ModelEndpointError> {
        let headers = self.inner_ref().request_headers(api_key);
        let body = self
            .inner_ref()
            .stream_request_body(messages, self.temperature())?;
        let request = client
            .post(self.inner_ref().completion_url())
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
        let headers = self.inner_ref().request_headers(api_key);
        let body = self
            .inner_ref()
            .fn_request_body(messages, function, self.temperature())?;
        let response = client
            .post(self.inner_ref().completion_url())
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let json = response.json().await?;
        Ok(self.inner_ref().handle_fn_response(json)?)
    }
}
