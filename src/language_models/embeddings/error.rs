use crate::errors::error_chain_fmt;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub type EmbeddingResult<T> = Result<T, EmbeddingError>;

#[derive(thiserror::Error)]
pub enum EmbeddingError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Json(#[from] serde_json::Error),
    Request(#[from] reqwest::Error),
}

impl Debug for EmbeddingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        error_chain_fmt(self, f)
    }
}

impl Display for EmbeddingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:?}", self)
    }
}
