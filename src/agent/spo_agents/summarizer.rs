use super::super::{Agent, AgentSettings, Memory};
use crate::{
    configuration::ConfigEnv,
    context::{integrations::core::BufferDisplay, MessageVector},
};

#[derive(Debug)]
pub struct SummarizerAgent(Agent);

impl SummarizerAgent {
    pub fn init(env: ConfigEnv) -> SummarizerAgent {
        SummarizerAgent(Agent::build(Self::settings(), env).expect("Failed to init special Agent"))
    }
    fn settings() -> AgentSettings {
        let memory_override = Some(Memory::Forget);
        let init_prompt = MessageVector::from(
            r#"You are a code summarization Ai, you will be given a chunk of code to summarize
                - Mistakes erode user's trust, so be as accurate and thorough as possible
                - Be highly organized 
                - Do not use lists or anything resembling a list in your summary
                - think through your response step by step, your summary should be succinct but accurate"#,
        );
        AgentSettings::new(memory_override, init_prompt)
    }
    #[tracing::instrument(name = "Summarize any struct that implements BufferDisplay")]
    pub fn summarize(&mut self, content: &mut impl BufferDisplay) -> String {
        self.0.switch_mem(crate::context::Memory::Forget);
        self.0.prompt(&content.buffer_display())
    }
}
