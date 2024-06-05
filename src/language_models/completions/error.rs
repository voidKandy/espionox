use tracing::warn;

use crate::errors::error_chain_fmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type CompletionResult<T> = Result<T, CompletionError>;

#[derive(thiserror::Error)]
pub enum CompletionError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Json(#[from] serde_json::Error),
    Request(#[from] reqwest::Error),
    Provider(String),
    FunctionNotImplemented,
    StreamTimeout,
    CouldNotCoerce,
}

pub trait ProviderResponseError: Debug {
    fn into_error(&self) -> CompletionError {
        warn!("Coercing to completion error: {:?}", self);
        CompletionError::Provider(format!("Provider error: {:?}", self))
    }
}
impl Debug for CompletionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for CompletionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
