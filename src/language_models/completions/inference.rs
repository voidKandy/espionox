use super::{
    error::{CompletionError, CompletionResult},
    functions::Function,
    streaming::ProviderStreamHandler,
    ModelParameters,
};
use crate::agents::memory::MessageStack;
use futures::Future;
use reqwest::{header::HeaderMap, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fmt::Debug, pin::Pin};

#[allow(unused)]
pub(crate) trait CompletionRequestBuilder: Debug + Sync + Send + 'static {
    fn model_str(&self) -> &str;
    fn url_str(&self) -> &str;
    fn serialize_messages(&self, stack: &MessageStack) -> Value;
    fn headers(&self, api_key: &str) -> HeaderMap;
    fn into_io_req(
        &self,
        stack: &MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Err(CompletionError::FunctionNotImplemented)
    }
    fn into_stream_req(
        &self,
        stack: &MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Err(CompletionError::FunctionNotImplemented)
    }
    // currently only supports single functions, but functions can technically be added to models
    // like messages
    fn serialize_function(
        &self,
        stack: &MessageStack,
        function: Function,
    ) -> CompletionResult<Value> {
        Err(CompletionError::FunctionNotImplemented)
    }
    fn process_function_response(&self, response_json: Value) -> CompletionResult<Value> {
        Err(CompletionError::FunctionNotImplemented)
    }
}

pub type ProcessResponseReturn<'r> =
    Pin<Box<dyn Future<Output = CompletionResult<CompletionResponse>> + Send + Sync + 'r>>;
pub trait CompletionRequest: Debug + Sync + Send + 'static {
    // We can't put Serialize and Deserialize as trait bounds, so we have `as_json`
    fn as_json(&self) -> CompletionResult<Value>;
    fn process_response<'r>(&'r self, response: Response) -> ProcessResponseReturn;
}

/// Any possible response from an inference endpoint
#[derive(Debug, Serialize, Deserialize)]
pub enum CompletionResponse {
    /// For IO completions
    Io(String),
    /// For streamed completions
    #[serde(skip)]
    Stream(ProviderStreamHandler),
    /// For function inference
    Function(Value),
}

impl From<String> for CompletionResponse {
    fn from(value: String) -> Self {
        Self::Io(value)
    }
}

impl From<ProviderStreamHandler> for CompletionResponse {
    fn from(value: ProviderStreamHandler) -> Self {
        Self::Stream(value)
    }
}

impl From<Value> for CompletionResponse {
    fn from(value: Value) -> Self {
        Self::Function(value)
    }
}

impl TryInto<String> for CompletionResponse {
    type Error = CompletionError;
    fn try_into(self) -> Result<String, Self::Error> {
        if let Self::Io(s) = self {
            return Ok(s);
        }
        Err(CompletionError::CouldNotCoerce)
    }
}

impl TryInto<ProviderStreamHandler> for CompletionResponse {
    type Error = CompletionError;
    fn try_into(self) -> Result<ProviderStreamHandler, Self::Error> {
        if let Self::Stream(s) = self {
            return Ok(s);
        }
        Err(CompletionError::CouldNotCoerce)
    }
}

impl TryInto<Value> for CompletionResponse {
    type Error = CompletionError;
    fn try_into(self) -> Result<Value, Self::Error> {
        if let Self::Function(s) = self {
            return Ok(s);
        }
        Err(CompletionError::CouldNotCoerce)
    }
}
