use crate::{agents::AgentError, errors::error_chain_fmt};

#[derive(thiserror::Error)]
pub enum MemoryError {
    Unexpected(#[from] anyhow::Error),
    Agent(#[from] AgentError),
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
