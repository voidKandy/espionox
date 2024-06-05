use reqwest_streams::error::StreamBodyError;

use crate::errors::error_chain_fmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type StreamResult<T> = Result<T, StreamError>;
#[derive(thiserror::Error)]
pub enum StreamError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Json(#[from] serde_json::Error),
    StreamBody(#[from] StreamBodyError),
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
        let display = match self {
            Self::Json(err) => err.to_string(),
            Self::Undefined(err) => err.to_string(),
            Self::StreamBody(err) => err.to_string(),
            Self::RetryError => "Retry Error".to_string(),
            Self::ReceiverTimeout => "Receiver Timeout".to_string(),
        };
        write!(f, "{}", display)
    }
}
