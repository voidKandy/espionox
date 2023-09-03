use super::{Agent, AgentSettings};
use crate::context::integrations::core::BufferDisplay;

#[derive(Debug)]
pub struct SummarizerAgent(Agent);

impl SummarizerAgent {
    #[tracing::instrument(name = "Create Summarizer Agent")]
    pub fn init() -> Self {
        SummarizerAgent(
            Agent::build(AgentSettings::summarizer())
                .expect("Failed to initialize summarizer agent"),
        )
    }

    #[tracing::instrument(name = "Summarize any struct that implements BufferDisplay")]
    pub fn summarize(&mut self, content: &mut impl BufferDisplay) -> String {
        self.0.switch_mem(crate::context::Memory::Forget);
        self.0.prompt(&content.buffer_display())
    }
}
