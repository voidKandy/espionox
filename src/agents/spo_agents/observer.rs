use crate::{
    agents::Agent,
    language_models::LanguageModel,
    memory::{CachingMechanism, Memory, Message, MessageRole, MessageVector, ToMessage},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObserver {
    agent: Box<Agent>,
    pub protocol: ObservationProtocol,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationProtocol {
    mutate_agent: Option<ObservationStep>,
    mutate_input: Option<ObservationStep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObservationStep {
    BeforePrompt,
    AfterPrompt,
    BeforeAndAfterPrompt,
}

impl ObservationProtocol {
    pub fn agent_mutator(&mut self, step: ObservationStep) {
        self.mutate_agent = Some(step);
    }

    pub fn input_mutator(&mut self, step: ObservationStep) {
        self.mutate_input = Some(step);
    }

    pub fn new() -> Self {
        Self {
            mutate_agent: None,
            mutate_input: None,
        }
    }
}

impl AgentObserver {
    pub fn new(protocol: ObservationProtocol) -> Self {
        Self {
            agent: Box::new(Self::agent()),
            protocol,
        }
    }

    pub fn from_agent(agent: Agent, protocol: ObservationProtocol) -> Self {
        Self {
            agent: Box::new(agent),
            protocol,
        }
    }

    pub fn inner_agent(&self) -> &Agent {
        self.agent.as_ref()
    }

    fn agent() -> Agent {
        let init_prompt = MessageVector::from_message(
            r#"You are an observation Ai, you will be prompted intermittently to evaluate the relationship between another agent's memory and prompts meant to be inputted into it.
                - When you generate output, your presence should be invisible. Never mention or reference your existance.
                - Never mention you are an AI or your limitations "#
        .to_string().to_message_with_role(MessageRole::System));

        let memory = Memory::build()
            .init_prompt(init_prompt)
            .caching_mechanism(CachingMechanism::SummarizeAtLimit {
                limit: 20,
                save_to_lt: false,
            })
            .finished();
        let model =
            LanguageModel::new_gpt(crate::language_models::openai::gpt::GptModel::Gpt3, 0.6);
        Agent {
            memory,
            model,
            ..Default::default()
        }
    }

    #[tracing::instrument(name = "Mutate input observer function")]
    #[async_recursion::async_recursion]
    pub async fn mutate_input(&mut self, context: &MessageVector, prompt: &str) -> String {
        let observer = &mut self.agent;
        let context_message = match context.len() > 0 { 
            true => format!("Here is the full context of the agent you're observing: [[BEGINNING OF AGENT CONTEXT]]{}[[END OF AGENT CONTEXT]].", context.to_string()),
            false => "The agent you are observing currently has an empty context.".to_string()
        };
        let prompt = format!(
            r#" {}
            The agent you are observing is to be prompted with this: [[BEGINNING OF PROMPT]]{}[[END OF PROMPT]]
            Given this information, please adjust the prompt considering the information the agent currently has access to within it's context."#,
            context_message,
            prompt
        );
        tracing::info!("Observer prompted...");
        let output = observer
            .prompt(prompt)
            .await
            .expect("Failed to prompt observer agent");
        tracing::info!("Observer output: {}", output);
        output
    }
    pub async fn mutate_agent(&mut self, agent: &mut Agent, context: MessageVector, prompt: &str) {
        unimplemented!()
    }
    pub fn has_pre_prompt_protocol(&self) -> bool {
        if self.protocol.mutate_agent == Some(ObservationStep::AfterPrompt)
            || self.protocol.mutate_input == Some(ObservationStep::AfterPrompt)
        {
            return false;
        }
        return true;
    }

    pub fn has_post_prompt_protocol(&self) -> bool {
        if self.protocol.mutate_agent == Some(ObservationStep::BeforePrompt)
            || self.protocol.mutate_input == Some(ObservationStep::BeforePrompt)
        {
            return false;
        }
        return true;
    }
}
