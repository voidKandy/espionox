use super::error::*;
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
use std::time::Duration;

pub trait EndpointCompletionModel: Sized + Clone + Copy {
    fn from_str(str: &str) -> Option<Self>;
    fn name(&self) -> &str;
    fn context_window(&self) -> i64;
}

pub trait EndpointCompletionHandler: Into<LLMCompletionHandler> {
    fn completion_url(&self) -> &str;
    fn model(&self) -> impl EndpointCompletionModel;
    fn temperature(&self) -> f32;
    fn request_headers(&self, api_key: &str) -> HeaderMap;
    fn io_request_body(&self, messages: &MessageStack) -> Value;
    fn fn_request_body(
        &self,
        messages: &MessageStack,
        function: Function,
    ) -> Result<Value, ModelEndpointError> {
        Err(ModelEndpointError::MethodUnimplemented)
    }
    fn stream_request_body(&self, messages: &MessageStack) -> Result<Value, ModelEndpointError> {
        Err(ModelEndpointError::MethodUnimplemented)
    }

    fn handle_io_response(&self, response: Value) -> Result<String, ModelEndpointError>;
    fn handle_fn_response(&self, response: Value) -> Result<Value, ModelEndpointError> {
        Err(ModelEndpointError::MethodUnimplemented)
    }
    // fn handle_stream_response(&mut self, response: Value) -> Result<String, ModelEndpointError> {
    //     Err(ModelEndpointError::MethodUnimplemented)
    // }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMCompletionHandler {
    OpenAi(OpenAiCompletionHandler),
}

impl LLMCompletionHandler {
    /// Returns immutable reference to inner Completion Handler
    pub fn inner_ref(&self) -> &impl EndpointCompletionHandler {
        match self {
            Self::OpenAi(h) => h,
        }
    }
    /// Returns mutable reference to inner Completion Handler
    pub fn inner_mut(&mut self) -> &mut impl EndpointCompletionHandler {
        match self {
            Self::OpenAi(h) => h,
        }
    }
    /// Creates LanguageModel with default gpt settings
    pub fn default_openai() -> Self {
        let openai = OpenAiCompletionHandler::default();
        Self::OpenAi(openai)
    }

    pub(crate) async fn get_io_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<String, ModelEndpointError> {
        let headers = self.inner_ref().request_headers(api_key);
        let body = self.inner_ref().io_request_body(messages);
        let response = client
            .post(self.inner_ref().completion_url())
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let json = response.json().await?;
        Ok(self.inner_ref().handle_io_response(json)?)
    }

    pub(crate) async fn get_stream_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
    ) -> Result<CompletionStream, ModelEndpointError> {
        let headers = self.inner_ref().request_headers(api_key);
        let body = self.inner_ref().stream_request_body(messages)?;
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

    pub(crate) async fn get_fn_completion(
        &self,
        messages: &MessageStack,
        api_key: &str,
        client: &Client,
        function: Function,
    ) -> Result<Value, ModelEndpointError> {
        let headers = self.inner_ref().request_headers(api_key);
        let body = self.inner_ref().fn_request_body(messages, function)?;
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
