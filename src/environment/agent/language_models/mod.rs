pub mod huggingface;
pub mod openai;
pub use huggingface::embed;
pub use openai::GptError;

use openai::gpt::{Gpt, GptResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguageModel {
    Gpt(Gpt),
}

impl From<Gpt> for LanguageModel {
    fn from(value: Gpt) -> Self {
        LanguageModel::Gpt(value)
    }
}

impl LanguageModel {
    pub fn completion_url(&self) -> &str {
        match self {
            Self::Gpt(_) => "https://api.openai.com/v1/chat/completions",
        }
    }

    pub fn io_completion_fn<'c>(
        &self,
    ) -> fn(
        &'c Client,
        &'c str,
        &'c Vec<Value>,
        &'c LanguageModel,
    ) -> Pin<Box<dyn Future<Output = Result<GptResponse, GptError>> + Send + 'c>> {
        openai::gpt::completions::completion_fn_wrapper
    }

    // Probably should create an into impl trait for this once more models are supported
    /// return a reference to the inner Gpt model struct
    pub fn inner_gpt(&self) -> Option<&Gpt> {
        match self {
            Self::Gpt(g) => Some(g),
            _ => None,
        }
    }
    /// Returns mutable reference to innner GPT
    pub fn inner_mut_gpt(&mut self) -> Option<&mut Gpt> {
        match self {
            Self::Gpt(g) => Some(g),
            _ => None,
        }
    }
    /// Creates LanguageModel with default gpt settings
    pub fn default_gpt() -> Self {
        let gpt = Gpt::default();
        Self::Gpt(gpt)
    }
}
