use reqwest::Client;
use serde_json::Value;

use crate::{agents::Agent, environment::agent_handle::CustomFunction};

use super::{
    memory::{Message, MessageStack},
    AgentError,
};

/// For when completions need to be gotten from outside of an environment
/// can be built from an environment or dispatch using `make_agent_independent` or
/// with `new`. Needs a `reqwest::Client` and valid api key
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndependentAgent {
    agent: Agent,
    #[serde(skip)]
    client: Client,
    api_key: String,
}

impl IndependentAgent {
    pub fn new(agent: Agent, client: Client, api_key: String) -> Self {
        Self {
            agent,
            client,
            api_key,
        }
    }

    pub fn mutate_agent_cache<F>(&mut self, f: F)
    where
        F: FnOnce(&mut MessageStack),
    {
        f(&mut self.agent.cache);
    }

    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, AgentError> {
        Ok(self
            .agent
            .completion_handler
            .get_embedding(text, &self.api_key, &self.client)
            .await?)
    }

    pub async fn io_completion(&self) -> Result<String, AgentError> {
        Ok(self
            .agent
            .completion_handler
            .get_io_completion(&self.agent.cache, &self.api_key, &self.client)
            .await?)
    }

    pub async fn function_completion(&self, function: CustomFunction) -> Result<Value, AgentError> {
        Ok(self
            .agent
            .completion_handler
            .get_fn_completion(
                &self.agent.cache,
                &self.api_key,
                &self.client,
                function.function(),
            )
            .await?)
    }
}
