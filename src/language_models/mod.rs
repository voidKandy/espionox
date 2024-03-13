#[cfg(feature = "bert")]
pub mod huggingface;

pub mod error;
pub mod openai;
use error::*;

use self::openai::endpoints::completions::{
    models::{OpenAi, OpenAiResponse},
    streaming::CompletionStream,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

use self::openai::functions::Function;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageModel {
    OpenAi(OpenAi),
}

impl From<OpenAi> for LanguageModel {
    fn from(value: OpenAi) -> Self {
        LanguageModel::OpenAi(value)
    }
}

impl LanguageModel {
    // Probably should create an into impl trait for this once more models are supported
    /// return a reference to the inner Gpt model struct
    pub fn inner_gpt(&self) -> Option<&OpenAi> {
        match self {
            Self::OpenAi(g) => Some(g),
        }
    }
    /// Returns mutable reference to innner GPT
    pub fn inner_mut_gpt(&mut self) -> Option<&mut OpenAi> {
        match self {
            Self::OpenAi(g) => Some(g),
        }
    }
    /// Creates LanguageModel with default gpt settings
    pub fn default_openai() -> Self {
        let openai = OpenAi::default();

        Self::OpenAi(openai)
    }

    pub(crate) fn io_completion_fn<'c>(
        &self,
    ) -> fn(
        &'c Client,
        &'c str,
        &'c Vec<Value>,
        &'c LanguageModel,
    ) -> Pin<
        Box<dyn Future<Output = Result<OpenAiResponse, ModelEndpointError>> + Send + Sync + 'c>,
    > {
        openai::endpoints::completions::io_completion_fn_wrapper
    }

    pub(crate) fn stream_completion_fn<'c>(
        &self,
    ) -> fn(
        &'c Client,
        &'c str,
        &'c Vec<Value>,
        &'c LanguageModel,
    ) -> Pin<
        Box<dyn Future<Output = Result<CompletionStream, ModelEndpointError>> + Send + Sync + 'c>,
    > {
        openai::endpoints::completions::stream_completion_fn_wrapper
    }

    pub(crate) fn function_completion_fn<'c>(
        &self,
    ) -> fn(
        &'c Client,
        &'c str,
        &'c Vec<Value>,
        &'c LanguageModel,
        &'c Function,
    ) -> Pin<
        Box<dyn Future<Output = Result<OpenAiResponse, ModelEndpointError>> + Send + Sync + 'c>,
    > {
        openai::endpoints::completions::function_completion_fn_wrapper
    }

    pub(crate) fn completion_url(&self) -> &str {
        match self {
            Self::OpenAi(_) => "https://api.openai.com/v1/chat/completions",
        }
    }
}
