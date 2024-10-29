use super::{
    super::{
        error::{CompletionResult, ProviderResponseError},
        inference::{CompletionRequest, CompletionRequestBuilder, CompletionResponse},
        ModelParameters,
    },
    builder::AnthropicCompletionModel,
    streaming::AnthropicStreamResponse,
};
use crate::agents::memory::{MessageRole, MessageStack};
use crate::language_models::completions::error::CompletionError;
use crate::language_models::completions::inference::ProcessResponseReturn;
use crate::language_models::completions::streaming::{
    CompletionStream, ProviderStreamHandler, StreamedCompletionHandler,
};
use futures::TryStreamExt;
use reqwest_streams::JsonStreamResponse;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct AnthropicIoRequest {
    pub model: String,
    pub messages: Value,
    pub temperature: f32,
    pub system: String,
    pub max_tokens: u32,
    pub stream: bool,
}

impl AnthropicIoRequest {
    pub fn new(
        stack: &MessageStack,
        params: &ModelParameters,
        typ: AnthropicCompletionModel,
        stream: bool,
    ) -> Self {
        let system_stack: MessageStack = stack.ref_filter_by(&MessageRole::System, true).into();
        let sans_system_stack: MessageStack =
            stack.ref_filter_by(&MessageRole::System, false).into();
        let system = system_stack
            .as_ref()
            .into_iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<&str>>()
            .join(".");
        let temperature = match params.temperature().ok() {
            Some(t) => t,
            None => 0.7,
        };
        Self {
            model: typ.model_str().to_string(),
            messages: typ.serialize_messages(&sans_system_stack),
            temperature,
            max_tokens: params.max_tokens.unwrap_or(1000),
            system,
            stream,
        }
    }
}

impl CompletionRequest for AnthropicIoRequest {
    fn as_json(&self) -> CompletionResult<Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn process_response<'r>(&'r self, response: reqwest::Response) -> ProcessResponseReturn<'r> {
        Box::pin(async move {
            match self.stream {
                false => {
                    let json = response.json().await?;
                    tracing::warn!("got response:  {json:#?}");
                    let response: AnthropicResponse = serde_json::from_value(json)?;
                    match response {
                        AnthropicResponse::Success(mut suc) => {
                            let content = suc.content.remove(0).text;
                            Ok(CompletionResponse::from(content))
                        }
                        AnthropicResponse::Err { error } => Err(error.into_error()),
                    }
                }
                true => {
                    let response_stream: CompletionStream = Box::new(
                        tokio::time::timeout(Duration::from_secs(10), async {
                            response
                                .json_array_stream::<Value>(1024)
                                .map_err(|err| err.into())
                        })
                        .await
                        .map_err(|_| CompletionError::StreamTimeout)?,
                    );
                    let handler: ProviderStreamHandler =
                        StreamedCompletionHandler::<AnthropicStreamResponse>::from(response_stream)
                            .into();
                    Ok(handler.into())
                }
            }
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum AnthropicResponse {
    Success(AnthropicSuccess),
    Err { error: AnthropicError },
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicSuccess {
    content: Vec<AnthropicResponseContent>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicError {
    message: String,
}
impl ProviderResponseError for AnthropicError {}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicResponseContent {
    text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnthropicUsage {
    input_tokens: i32,
    output_tokens: i32,
}
