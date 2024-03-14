#[cfg(feature = "bert")]
pub mod huggingface;

pub mod anthropic;

pub mod endpoint_completions;
pub mod error;
pub mod openai;

pub enum ApiKey {
    OpenAi(String),
    Anthropic(String),
}
