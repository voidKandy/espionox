use futures_util::Future;

use crate::language_models::endpoint_completions::EndpointCompletionHandler;

use super::{Dispatch, EnvMessage};

use std::pin::Pin;

use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};

pub mod error {
    use crate::agents::AgentError;
    use crate::errors::error_chain_fmt;
    use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

    #[derive(thiserror::Error)]
    pub enum ListenerError {
        #[error(transparent)]
        Undefined(#[from] anyhow::Error),
        Agent(#[from] AgentError),
        IncorrectTrigger,
        NoAgent,
        Other(String),
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

pub type ListenerMethodReturn<'l> =
    Pin<Box<dyn Future<Output = Result<EnvMessage, ListenerError>> + Send + Sync + 'l>>;

pub trait EnvListener<H>: std::fmt::Debug + Send + Sync + 'static
where
    H: EndpointCompletionHandler,
{
    /// Returns Some when the listener should be triggered
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage>;
    /// method to be called when listener is activated, must return an env message to replace input
    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch<H>,
    ) -> ListenerMethodReturn;
}

pub(crate) async fn run_listeners<H: EndpointCompletionHandler>(
    mut message: EnvMessage,
    listeners: Arc<RwLock<Vec<Box<dyn EnvListener<H>>>>>,
    mut dispatch: &mut RwLockWriteGuard<'_, Dispatch<H>>,
) -> Result<EnvMessage, ListenerError> {
    let mut listeners_write = listeners.write().await;
    let mut active_listeners = listeners_write.iter_mut().fold(vec![], |mut active, l| {
        if l.trigger(&message).is_some() {
            active.push(l)
        }
        active
    });
    for l in active_listeners.iter_mut() {
        message = l.method(message, &mut dispatch).await?;
    }
    Ok(message)
}
