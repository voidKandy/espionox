use super::agents::{Agent, SpecialAgent};
use super::context::config::Context;
use super::functions::config::Function;
use serde_json::Value;
use std::error::Error;

pub struct AgentHandler {
    pub special_agent: SpecialAgent,
    pub agent: Agent,
    pub context: Context,
}

pub trait Operations {
    fn infer_problem() {}
    fn parse_response(&self, response: Value) -> Option<Vec<String>>;
}

impl AgentHandler {
    pub fn new(special_agent: SpecialAgent) -> AgentHandler {
        AgentHandler {
            special_agent: special_agent.clone(),
            agent: special_agent.init_agent(),
            context: Context::new(Some(&special_agent.get_sys_prompt())),
        }
    }
    pub async fn prompt(&self) -> Result<String, Box<dyn Error>> {
        self.agent
            .get_completion_response(&self.context.messages)
            .await
    }
    pub async fn function_prompt(&self, function: &Function) -> Result<Value, Box<dyn Error>> {
        self.agent
            .get_function_completion_response(&self.context.messages, &function)
            .await
    }
}

impl Operations for AgentHandler {
    fn infer_problem() {}
    fn parse_response(&self, response: Value) -> Option<Vec<String>> {
        let arguments = response.get("arguments")?.as_str()?;
        let parsed_arguments = serde_json::from_str::<Value>(arguments).ok()?;
        let commands = parsed_arguments.get("commands")?.as_array()?;
        let command_strings = commands
            .iter()
            .filter_map(|command| command.as_str().map(String::from))
            .collect::<Vec<String>>();
        Some(command_strings)
    }
}
