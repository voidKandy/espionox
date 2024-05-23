use super::{
    anthropic::AnthropicCompletionHandler, error::*, openai::embeddings::OpenAiEmbeddingModel,
    ModelProvider,
};

use crate::agents::memory::MessageStack;

use super::openai::completions::{functions::Function, OpenAiCompletionHandler};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;

pub trait InferenceEndpointHandler: Debug + Sync + Send + 'static {
    fn name(&self) -> &str;
    fn completion_url(&self) -> &str;
    fn request_headers(&self, api_key: &str) -> HeaderMap;
}

pub trait CompletionEndpointHandler: InferenceEndpointHandler {
    fn context_window(&self) -> i64;
    fn agent_cache_to_json(&self, cache: &MessageStack) -> Vec<Value>;
    fn io_request_body(&self, messages: &MessageStack, temperature: f32) -> Value;
    fn fn_request_body(
        &self,
        _messages: &MessageStack,
        _function: Function,
    ) -> Result<Value, InferenceHandlerError> {
        Err(InferenceHandlerError::MethodUnimplemented)
    }
    fn stream_request_body(
        &self,
        _messages: &MessageStack,
        _temperature: f32,
    ) -> Result<Value, InferenceHandlerError> {
        Err(InferenceHandlerError::MethodUnimplemented)
    }

    fn handle_io_response(&self, response: Value) -> Result<String, InferenceHandlerError>;
    fn handle_fn_response(&self, _response: Value) -> Result<Value, InferenceHandlerError> {
        Err(InferenceHandlerError::MethodUnimplemented)
    }
}

pub trait EmbeddingEndpointHandler: InferenceEndpointHandler {
    fn request_body(&self, _text: &str) -> Value;
    fn handle_response(&self, _response: Value) -> Result<Vec<f32>, InferenceHandlerError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMInferenceHandler {
    Completion(LLMCompletionHandler),
    Embedding(LLMEmbeddingHandler),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMCompletionHandler {
    OpenAi(OpenAiCompletionHandler),
    Anthropic(AnthropicCompletionHandler),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLMEmbeddingHandler {
    OpenAi(OpenAiEmbeddingModel),
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

impl From<OpenAiEmbeddingModel> for LLMEmbeddingHandler {
    fn from(value: OpenAiEmbeddingModel) -> Self {
        Self::OpenAi(value)
    }
}

impl LLMCompletionHandler {
    fn inner(self) -> Box<dyn CompletionEndpointHandler> {
        match self {
            Self::OpenAi(h) => Box::new(h),
            Self::Anthropic(h) => Box::new(h),
        }
    }
}

impl LLMEmbeddingHandler {
    fn inner(self) -> Box<dyn EmbeddingEndpointHandler> {
        match self {
            Self::OpenAi(h) => Box::new(h),
        }
    }
}

impl LLMInferenceHandler {
    pub(crate) fn provider(&self) -> ModelProvider {
        match self {
            Self::Embedding(h) => match h {
                LLMEmbeddingHandler::OpenAi(_) => ModelProvider::OpenAi,
            },
            Self::Completion(h) => match h {
                LLMCompletionHandler::OpenAi(_) => ModelProvider::OpenAi,
                LLMCompletionHandler::Anthropic(_) => ModelProvider::Anthropic,
            },
        }
    }
    pub(crate) fn comp_handler(
        &self,
    ) -> Result<Box<dyn CompletionEndpointHandler>, InferenceHandlerError> {
        if let LLMInferenceHandler::Completion(h) = self {
            return Ok(h.clone().inner());
        }
        Err(InferenceHandlerError::IncorrectHandler)
    }

    pub(crate) fn emb_handler(
        &self,
    ) -> Result<Box<dyn EmbeddingEndpointHandler>, InferenceHandlerError> {
        if let LLMInferenceHandler::Embedding(h) = self {
            return Ok(h.clone().inner());
        }
        Err(InferenceHandlerError::IncorrectHandler)
    }
}

impl From<LLMCompletionHandler> for LLMInferenceHandler {
    fn from(value: LLMCompletionHandler) -> Self {
        Self::Completion(value)
    }
}

impl From<LLMEmbeddingHandler> for LLMInferenceHandler {
    fn from(value: LLMEmbeddingHandler) -> Self {
        Self::Embedding(value)
    }
}
