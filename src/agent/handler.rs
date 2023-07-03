use super::agents::SpecialAgent;
use super::api::gpt::Gpt;
use super::context::config::Context;
use super::functions::config::Function;
use std::error::Error;

pub struct AgentHandler {
    pub special_agent: SpecialAgent,
    pub gpt: Gpt,
    pub context: Context,
}

impl AgentHandler {
    pub fn new(special_agent: SpecialAgent) -> AgentHandler {
        AgentHandler {
            special_agent: special_agent.clone(),
            gpt: special_agent.get_gpt(),
            context: Context::new(Some(&special_agent.get_sys_prompt())),
        }
    }
    pub async fn prompt(&mut self) -> Result<String, Box<dyn Error>> {
        match self
            .gpt
            .completion(&self.context.messages)
            .await?
            .parse_response()
        {
            Ok(content) => {
                self.context.append_to_messages("assistant", &content);
                Ok(content)
            }
            Err(err) => Err(err),
        }
    }
    pub async fn function_prompt(
        &mut self,
        function: &Function,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        match self
            .gpt
            .function_completion(&self.context.messages, &function)
            .await?
            .parse_fn_response(&function.perameters.properties[0].name)
        {
            Ok(content) => {
                content.clone().into_iter().for_each(|c| {
                    self.context.append_to_messages("assistant", &c);
                });
                Ok(content)
            }
            Err(err) => Err(err),
        }
    }
}
