#[cfg(feature = "bert")]
pub mod huggingface;

pub mod anthropic;

pub mod endpoint_completions;
pub mod error;
pub mod openai;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ModelProvider {
    OpenAi,
    Anthropic,
}
