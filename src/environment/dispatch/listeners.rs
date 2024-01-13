use futures_util::Future;

use crate::environment::errors::DispatchError;

use super::{Dispatch, EnvMessage};
use std::pin::Pin;

pub trait EnvListener: std::fmt::Debug + Send + Sync + 'static {
    /// Takes EnvMessage and returns an option containing a reference to it if the Listener is triggered
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage>;
    /// method to be called when listener is activated
    fn method<'l>(
        &'l self,
        trigger_message: &'l EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> Pin<Box<dyn Future<Output = Result<(), DispatchError>> + Send + Sync + 'l>>;
}
