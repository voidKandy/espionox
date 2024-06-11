use super::{
    super::inference::CompletionRequest, builder::OpenAiCompletionModel,
    streaming::OpenAiStreamResponse,
};
use crate::{
    agents::memory::MessageStack,
    language_models::completions::{
        error::{CompletionError, CompletionResult, ProviderResponseError},
        inference::{CompletionRequestBuilder, CompletionResponse, ProcessResponseReturn},
        streaming::{CompletionStream, ProviderStreamHandler, StreamedCompletionHandler},
        ModelParameters,
    },
};
use anyhow::anyhow;
use futures::TryStreamExt;
use reqwest::Response;
use reqwest_streams::JsonStreamResponse;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::info;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct OpenAiIoRequest {
    pub model: String,
    pub messages: Value,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
    pub n: u32,
}

impl OpenAiIoRequest {
    pub fn new(
        stack: &MessageStack,
        params: &ModelParameters,
        typ: OpenAiCompletionModel,
        stream: bool,
    ) -> Self {
        let temperature = match params.temperature().ok() {
            Some(t) => t,
            None => 0.7,
        };
        OpenAiIoRequest {
            model: typ.model_str().to_string(),
            messages: CompletionRequestBuilder::serialize_messages(&typ, stack),
            temperature,
            stream,
            max_tokens: params.max_tokens.unwrap_or(1000),
            n: params.n.unwrap_or(1),
        }
    }
}

impl CompletionRequest for OpenAiIoRequest {
    fn as_json(&self) -> CompletionResult<Value> {
        Ok(serde_json::to_value(self)?)
    }

    fn process_response<'r>(&'r self, response: Response) -> ProcessResponseReturn<'r> {
        Box::pin(async move {
            match self.stream {
                false => {
                    let json = response.json().await?;
                    let response: OpenAiResponse = serde_json::from_value(json)?;
                    return match response {
                        OpenAiResponse::Success(mut suc) => {
                            let content = suc.choices.remove(0).message.content.ok_or(
                                CompletionError::from(anyhow!("No content in success message")),
                            )?;
                            Ok(CompletionResponse::from(content))
                        }
                        OpenAiResponse::Err { error } => Err(error.into_error()),
                    };
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
                        StreamedCompletionHandler::<OpenAiStreamResponse>::from(response_stream)
                            .into();
                    Ok(handler.into())
                }
            }
        })
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum OpenAiResponse {
    Success(OpenAiSuccess),
    Err { error: OpenAiErr },
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct OpenAiSuccess {
    pub usage: OpenAiUsage,
    pub choices: Vec<Choice>,
}

impl ProviderResponseError for OpenAiErr {}
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct OpenAiErr {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Choice {
    pub message: GptMessage,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct GptMessage {
    pub role: String,
    pub content: Option<String>,
    pub function_call: Option<Value>,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct OpenAiUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: Option<i32>,
    pub total_tokens: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn openai_response_parsed_correctly() {
        let value = json!(
        {
            "id": "chatcmpl-abc123",
            "object": "chat.completion",
            "created": 1677858242,
            "model": "gpt-3.5-turbo-0613",
            "usage": {
                "prompt_tokens": 13,
                "completion_tokens": 7,
                "total_tokens": 20
            },
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "\n\nThis is a test!"
                    },
                    "logprobs": null,
                    "finish_reason": "stop",
                    "index": 0
                }
            ]
        });

        let res: OpenAiResponse = serde_json::from_value(value).unwrap();
        let expected = OpenAiResponse::Success(OpenAiSuccess {
            usage: OpenAiUsage {
                prompt_tokens: 13,
                completion_tokens: Some(7),
                total_tokens: 20,
            },
            choices: vec![{
                Choice {
                    message: GptMessage {
                        role: "assistant".to_string(),
                        content: Some("\n\nThis is a test!".to_string()),
                        function_call: None,
                    },
                }
            }],
        });
        assert_eq!(res, expected);
    }
}
