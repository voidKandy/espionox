use crate::errors::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum GptError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Completion(#[from] reqwest::Error),
    NoApiKey,
    Recoverable(String),
}

impl std::fmt::Debug for GptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for GptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
