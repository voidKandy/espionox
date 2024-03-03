use reqwest::Client;
use serde_json::Value;

use crate::{agents::Agent, environment::agent_handle::CustomFunction};

use super::AgentError;

/// For when completions need to be gotten from outside of an environment
/// can be built from an environment or dispatch using `make_agent_independent` or
/// with `new`. Needs a `reqwest::Client` and valid api key
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndependentAgent {
    pub agent: Agent,
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

    pub async fn io_completion(&mut self) -> Result<String, AgentError> {
        let func = self.agent.model.io_completion_fn();
        let context = (&self.agent.cache).into();
        let response = func(&self.client, &self.api_key, &context, &self.agent.model).await?;
        self.agent.handle_completion_response(response)
    }

    pub async fn function_completion(
        &mut self,
        function: CustomFunction,
    ) -> Result<Value, AgentError> {
        let func = self.agent.model.function_completion_fn();
        let context = (&self.agent.cache).into();
        let response = func(
            &self.client,
            &self.api_key,
            &context,
            &self.agent.model,
            &function.function(),
        )
        .await?;
        Ok(response.parse_fn()?)
    }
}
