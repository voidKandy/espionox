use crate::{
    agent::{Agent, AgentError},
    context::memory::{CachingMechanism, Memory, MessageRole, MessageVector, ToMessage},
    language_models::LanguageModel,
};

#[derive(Debug)]
pub struct SummarizerAgent(Agent);

impl SummarizerAgent {
    pub fn init() -> SummarizerAgent {
        let init_prompt = MessageVector::from_message(
            r#"You are a code summarization Ai, you will be given a chunk of code to summarize
                - Mistakes erode user's trust, so be as accurate and thorough as possible
                - Be highly organized 
                - Do not use lists or anything resembling a list in your summary
                - think through your response step by step, your summary should be succinct but accurate"#
        .to_string().to_message(MessageRole::System));

        let memory = Memory::build()
            .init_prompt(init_prompt)
            .caching_mechanism(CachingMechanism::Forgetful)
            .finished();
        let model = LanguageModel::default_gpt();
        SummarizerAgent(Agent { memory, model })
    }
    #[tracing::instrument(name = "Summarize anything that implements ToMessage")]
    pub async fn summarize(content: impl ToMessage) -> Result<String, AgentError> {
        Self::init().0.prompt(content).await
    }
}
