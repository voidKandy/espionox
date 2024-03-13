use crate::errors::error_chain_fmt;
use crate::language_models::error::ModelEndpointError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(thiserror::Error)]
pub enum AgentError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    ModelError(#[from] ModelEndpointError),
    EnvSend,
    AgentSenderIsNone,
}

impl Debug for AgentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for AgentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
