pub use super::agent::language_models::GptError;
use super::{
    agent::language_models::openai::gpt::streaming_utils::StreamError, dispatch::ListenerError,
};
use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum EnvError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Listener(#[from] ListenerError),
    Dispatch(#[from] DispatchError),
    Join(#[from] tokio::task::JoinError),
    Timeout(#[from] tokio::time::error::Elapsed),
    Request(String),
    Send,
}

#[derive(thiserror::Error)]
pub enum DispatchError {
    Undefined(#[from] anyhow::Error),
    Gpt(#[from] GptError),
    Agent(#[from] AgentError),
    Stream(#[from] StreamError),
    Timeout(#[from] tokio::time::error::Elapsed),
    NoApiKey,
    AgentIsNone,
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

impl std::fmt::Debug for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for DispatchError {
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
