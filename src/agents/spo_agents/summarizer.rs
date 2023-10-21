use crate::{
    agents::{Agent, AgentError},
    language_models::LanguageModel,
    memory::{CachingMechanism, Memory, MessageRole, MessageVector, ToMessage},
};

// #[derive(Debug)]
// pub struct SummarizerAgent(Agent);

#[derive(Debug)]
pub enum SummarizerAgent {
    General,
    Memory,
}

impl SummarizerAgent {
    pub fn agent(&self) -> Agent {
        match self {
            Self::General => {
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
                Agent { memory, model }
            }
            Self::Memory => {
                let init_prompt = MessageVector::from_message(
                    r#"You are a memory summarizer agent, given the message thread, 
            your task is to provide a concise and informative summary of the 
            conversation. Consider the user's input and your responses. 
            Be accurate, organized, and include key points from the dialogue. 
            Your summaries should be helpful and reflect the essence of the 
            conversation. Remember, clarity and brevity are essential."#
                        .to_string()
                        .to_message(MessageRole::System),
                );

                let memory = Memory::build()
                    .init_prompt(init_prompt)
                    .caching_mechanism(CachingMechanism::Forgetful)
                    .finished();
                let model = LanguageModel::default_gpt();
                Agent { memory, model }
            }
        }
    }
    #[tracing::instrument(name = "Summarize anything that implements ToMessage")]
    pub async fn summarize(content: impl ToMessage) -> Result<String, AgentError> {
        Self::General.agent().prompt(content).await
    }
    #[tracing::instrument(name = "Summarize MessageVector")]
    pub async fn summarize_memory(memory: MessageVector) -> Result<String, AgentError> {
        let mut agent = Self::Memory.agent();
        tracing::info!("Summarizer initialized");
        let summary = agent
            .prompt(memory.to_string())
            .await
            .expect("Failed to get summary");
        Ok(format!(
            "Here is a summary of a chunk of the current conversation: {}",
            summary
        ))
    }
}
