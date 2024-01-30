use futures_util::Future;

use crate::environment::errors::{AgentError, DispatchError, EnvError};

use super::{Dispatch, EnvMessage};
use std::pin::Pin;

use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::errors::error_chain_fmt;

pub type ListenerMethodReturn<'l> =
    Pin<Box<dyn Future<Output = Result<(), ListenerError>> + Send + Sync + 'l>>;

pub trait EnvListener: std::fmt::Debug + Send + Sync + 'static {
    /// Returns Some when the listener should be triggered
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage>;
    /// method to be called when listener is activated
    fn method<'l>(
        &'l mut self,
        trigger_message: &'l EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn;
    fn mutate<'l>(&'l mut self, origin: EnvMessage) -> EnvMessage {
        origin
    }
}

pub(crate) async fn run_listeners(
    mut message: EnvMessage,
    listeners: Arc<RwLock<Vec<Box<dyn EnvListener>>>>,
    mut dispatch: &mut RwLockWriteGuard<'_, Dispatch>,
) -> Result<EnvMessage, ListenerError> {
    let mut listeners_write = listeners.write().await;
    let mut active_listeners = listeners_write.iter_mut().fold(vec![], |mut active, l| {
        if l.trigger(&message).is_some() {
            active.push(l)
        }
        active
    });
    for l in active_listeners.iter_mut() {
        l.method(&message, &mut dispatch).await?;
        message = l.mutate(message);
    }
    Ok(message)
}

#[derive(thiserror::Error)]
pub enum ListenerError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    Dispatch(#[from] DispatchError),
    Agent(#[from] AgentError),
    Other(String),
}

impl std::fmt::Debug for ListenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for ListenerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
