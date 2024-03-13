use crate::errors::error_chain_fmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(thiserror::Error)]
pub enum ModelEndpointError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    NetRequest(#[from] reqwest::Error),
    Recoverable,
    NoApiKey,
}

impl Debug for ModelEndpointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for ModelEndpointError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
