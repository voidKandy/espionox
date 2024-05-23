use std::pin::Pin;

use futures::Future;

use super::{error::AgentResult, Agent};
pub mod error {
    use crate::errors::error_chain_fmt;
    use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

    #[derive(thiserror::Error)]
    pub enum ListenerError {
        #[error(transparent)]
        Undefined(#[from] anyhow::Error),
        NoMethod,
    }

    impl Debug for ListenerError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            error_chain_fmt(self, f)
        }
    }

    impl Display for ListenerError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            write!(f, "{:?}", self)
        }
    }
}
use error::*;

#[derive(Debug, PartialEq, Eq)]
pub enum ListenerTrigger {
    String(String),
    Int(i64),
}

impl From<String> for ListenerTrigger {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for ListenerTrigger {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<i64> for ListenerTrigger {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

pub type ListenerCallReturn<'l> = Pin<Box<dyn Future<Output = AgentResult<()>> + Send + Sync + 'l>>;
/// Contains async and sync methods, Both COULD be implemented but that is not reccomended. If both
/// are implemented, sync will always execute first
pub trait AgentListener: std::fmt::Debug + Send + Sync + 'static {
    fn trigger<'l>(&self) -> ListenerTrigger;
    /// needs to be wrapped in `Box::pin(async move {})`
    fn async_method<'l>(&'l mut self, _a: &'l mut Agent) -> ListenerCallReturn<'l> {
        Box::pin(async move { Err(ListenerError::NoMethod.into()) })
    }
    fn sync_method<'l>(&'l mut self, _a: &'l mut Agent) -> AgentResult<()> {
        Err(ListenerError::NoMethod.into())
    }
}
