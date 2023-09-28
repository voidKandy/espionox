use super::super::{Agent, AgentSettings};
use crate::{
    agent::AgentError,
    context::{integrations::core::BufferDisplay, short_term::ShortTermMemory, MessageVector},
};

#[derive(Debug)]
pub struct SummarizerAgent(Agent);

impl SummarizerAgent {
    pub fn init() -> SummarizerAgent {
        SummarizerAgent(Agent::build(Self::settings()).expect("Failed to init special Agent"))
    }
    fn settings() -> AgentSettings {
        let stm = ShortTermMemory::Forget;
        let init_prompt = MessageVector::init_with_system_prompt(
            r#"You are a code summarization Ai, you will be given a chunk of code to summarize
                - Mistakes erode user's trust, so be as accurate and thorough as possible
                - Be highly organized 
                - Do not use lists or anything resembling a list in your summary
                - think through your response step by step, your summary should be succinct but accurate"#,
        );
        AgentSettings::new()
            .init_prompt(init_prompt)
            .short_term(stm)
            .finish()
    }
    #[tracing::instrument(name = "Summarize any struct that implements BufferDisplay")]
    pub async fn summarize(content: impl BufferDisplay) -> Result<String, AgentError> {
        Self::init().0.prompt(content).await
    }
}
