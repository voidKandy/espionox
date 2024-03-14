use reqwest::Client;
use serde_json::Value;

use crate::{
    agents::Agent, environment::agent_handle::CustomFunction,
    language_models::endpoint_completions::EndpointCompletionHandler,
};

use super::AgentError;

/// For when completions need to be gotten from outside of an environment
/// can be built from an environment or dispatch using `make_agent_independent` or
/// with `new`. Needs a `reqwest::Client` and valid api key
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndependentAgent<H: EndpointCompletionHandler> {
    pub agent: Agent<H>,
    #[serde(skip)]
    client: Client,
    api_key: String,
}

impl<H: EndpointCompletionHandler> IndependentAgent<H> {
    pub fn new(agent: Agent<H>, client: Client, api_key: String) -> Self {
        Self {
            agent,
            client,
            api_key,
        }
    }

    pub async fn io_completion(&mut self) -> Result<String, AgentError> {
        Ok(self
            .agent
            .completion_handler
            .get_io_completion(&self.agent.cache, &self.api_key, &self.client)
            .await?)
    }

    pub async fn function_completion(
        &mut self,
        function: CustomFunction,
    ) -> Result<Value, AgentError> {
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
