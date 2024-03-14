pub mod error;
pub mod independent;
pub mod memory;
use std::fmt::Debug;

use memory::MessageStack;

use crate::language_models::endpoint_completions::{
    EndpointCompletionHandler, LLMCompletionHandler,
};
pub use error::AgentError;

/// Agent struct for interracting with LLM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent<H: EndpointCompletionHandler> {
    pub cache: MessageStack,
    pub completion_handler: LLMCompletionHandler<H>,
}

impl<H: EndpointCompletionHandler> Agent<H> {
    /// For creating an Agent given system prompt content and model
    pub fn new(init_prompt: &str, completion_handler: LLMCompletionHandler<H>) -> Self {
        let cache = MessageStack::new(init_prompt);
        Agent {
            cache,
            completion_handler,
        }
    }
}
