pub mod error;
pub mod independent;
pub mod memory;
use std::fmt::Debug;

use memory::MessageStack;

use crate::language_models::{ModelProvider, LLM};
pub use error::AgentError;

/// Agent struct for interracting with LLM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub cache: MessageStack,
    pub(crate) completion_handler: LLM,
}

impl Agent {
    /// For creating an Agent given optional system prompt content and model
    pub fn new(init_prompt: Option<&str>, completion_handler: LLM) -> Self {
        let cache = match init_prompt {
            Some(p) => MessageStack::new(p),
            None => MessageStack::init(),
        };
        Agent {
            cache,
            completion_handler,
        }
    }

    pub fn provider(&self) -> ModelProvider {
        self.completion_handler.provider()
    }
}
