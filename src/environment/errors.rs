pub use super::agent::language_models::GptError;
use super::EnvMessage;
use crate::errors::error_chain_fmt;
use tokio::sync::mpsc::error::SendError;

#[derive(thiserror::Error)]
pub enum EnvError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Agent(#[from] AgentError),
    Gpt(#[from] GptError),
    Request(String),
    Send,
}

#[derive(thiserror::Error)]
pub enum AgentError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    GptError(#[from] GptError),
    EnvSend,
    AgentSenderIsNone,
}

#[derive(thiserror::Error)]
pub enum MemoryError {
    Unexpected(#[from] anyhow::Error),
    Agent(#[from] AgentError),
}

impl std::fmt::Debug for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Debug for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
