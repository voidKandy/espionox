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

pub trait EndpointCompletionHandler: Debug + Sync + Send + 'static {
    fn name(&self) -> &str;
    fn agent_cache_to_json(&self, cache: &MessageStack) -> Vec<Value>;
    fn context_window(&self) -> i64;
    fn completion_url(&self) -> &str;
    fn request_headers(&self, api_key: &str) -> HeaderMap;
    fn io_request_body(&self, messages: &MessageStack, temperature: f32) -> Value;
    fn fn_request_body(
        &self,
        _messages: &MessageStack,
        _function: Function,
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
pub enum LLMCompletionHandler {
    OpenAi(OpenAiCompletionHandler),
    Anthropic(AnthropicCompletionHandler),
}

impl From<OpenAiCompletionHandler> for LLMCompletionHandler {
    fn from(value: OpenAiCompletionHandler) -> Self {
        Self::OpenAi(value)
    }
}

impl From<AnthropicCompletionHandler> for LLMCompletionHandler {
    fn from(value: AnthropicCompletionHandler) -> Self {
        Self::Anthropic(value)
    }
}

impl LLMCompletionHandler {
    fn inner(self) -> Box<dyn EndpointCompletionHandler> {
        match self {
            Self::OpenAi(h) => Box::new(h),
            Self::Anthropic(h) => Box::new(h),
        }
    }

    pub(crate) fn comp_handler(&self) -> Box<dyn EndpointCompletionHandler> {
        self.clone().inner()
    }
}

// impl AsRef<Box<dyn EndpointCompletionHandler>> for LLMCompletionHandler {
//     fn as_ref(&self) -> Box<&dyn EndpointCompletionHandler> {
//         match self {
//             Self::OpenAi(h) => Box::new(h as dyn EndpointCompletionHandler),
//             Self::Anthropic(h) => Box::new(h as dyn EndpointCompletionHandler),
//         }
//     }
// }
