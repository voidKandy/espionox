pub mod error;
pub mod memory;
use crate::language_models::completions::{
    functions::Function, streaming::ProviderStreamHandler, CompletionModel,
};
pub use error::AgentError;
use memory::MessageStack;
use std::fmt::Debug;

use error::AgentResult;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub cache: MessageStack,
    pub completion_model: CompletionModel,
}

impl Agent {
    /// For creating an Agent given optional system prompt content and model
    pub fn new(init_prompt: Option<&str>, completion_model: CompletionModel) -> Self {
        let cache = match init_prompt {
            Some(p) => MessageStack::new(p),
            None => MessageStack::init(),
        };
        Agent {
            cache,
            completion_model,
        }
    }

    /// Get a simple string response from a model
    pub async fn io_completion(&mut self) -> AgentResult<String> {
        Ok(self.completion_model.get_io_completion(&self.cache).await?)
    }

    /// Get a streamed response from a model
    pub async fn stream_completion(&mut self) -> AgentResult<ProviderStreamHandler> {
        let cs = self
            .completion_model
            .get_stream_completion(&self.cache)
            .await?;

        Ok(cs.into())
    }

    /// Get a function completion from a model, returns a JSON object
    pub async fn function_completion(
        &mut self,
        function: Function,
    ) -> AgentResult<serde_json::Value> {
        Ok(self
            .completion_model
            .get_fn_completion(&self.cache, function)
            .await?)
    }
}
