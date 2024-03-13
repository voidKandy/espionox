use crate::errors::error_chain_fmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(thiserror::Error)]
pub enum StreamError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    EnvMessageSenderFail,
    ReceiverTimeout,
    RetryError,
}

impl Debug for StreamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for StreamError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
