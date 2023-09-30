use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum AgentError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    GptError(#[from] crate::language_models::openai::gpt::GptError),
    RetryError,
}

impl std::fmt::Debug for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
