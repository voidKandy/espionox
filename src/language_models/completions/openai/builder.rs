use super::{
    super::inference::{CompletionRequest, CompletionRequestBuilder},
    requests::{OpenAiFunctionRequest, OpenAiIoRequest},
};
use crate::language_models::completions::{error::CompletionResult, ModelParameters};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub enum OpenAiCompletionModel {
    #[default]
    Gpt3,
    Gpt4,
}

const GPT3_MODEL_STR: &str = "gpt-3.5-turbo-0125";
const GPT4_MODEL_STR: &str = "gpt-4-0125-preview";

impl CompletionRequestBuilder for OpenAiCompletionModel {
    fn model_str(&self) -> &str {
        match self {
            Self::Gpt3 => GPT3_MODEL_STR,
            Self::Gpt4 => GPT4_MODEL_STR,
        }
    }

    fn url_str(&self) -> &str {
        "https://api.openai.com/v1/chat/completions"
    }

    fn headers(&self, api_key: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        map.insert("Content-Type", "application/json".parse().unwrap());
        map
    }

    fn serialize_messages(&self, stack: &crate::agents::memory::MessageStack) -> Value {
        stack
            .as_ref()
            .to_owned()
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<Value>>()
            .into()
    }

    fn into_io_req(
        &self,
        stack: &crate::agents::memory::MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(OpenAiIoRequest::new(stack, params, *self, false)))
    }
    fn into_stream_req(
        &self,
        stack: &crate::agents::memory::MessageStack,
        params: &ModelParameters,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(OpenAiIoRequest::new(stack, params, *self, true)))
    }
    fn into_function_req(
        &self,
        stack: &crate::agents::memory::MessageStack,
        // Becuase of the complexity of function calls, we'll just use raw function bodies
        // Hopefully this changes in the future
        function_body: Value,
        function_name: &str,
    ) -> CompletionResult<Box<dyn CompletionRequest>> {
        Ok(Box::new(OpenAiFunctionRequest::new(
            stack,
            *self,
            function_body,
            function_name,
        )))
    }
}
