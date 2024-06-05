pub mod actions;
pub mod error;
pub mod listeners;
pub mod memory;
use crate::language_models::completions::CompletionModel;
pub use error::AgentError;
use memory::MessageStack;
use std::{fmt::Debug, future::Future};
use tracing::warn;

use self::{
    error::AgentResult,
    listeners::{AgentListener, ListenerTrigger},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub cache: MessageStack,
    pub(crate) completion_model: CompletionModel,
    #[serde(skip)]
    /// Essentially callbacks that optionally trigger on the `do_action` method
    listeners: Vec<Box<dyn listeners::AgentListener>>,
}

impl Agent {
    /// For creating an Agent given optional system prompt content and model
    pub fn new(init_prompt: Option<&str>, completion_model: CompletionModel) -> Self {
        let cache = match init_prompt {
            Some(p) => MessageStack::new(p),
            None => MessageStack::init(),
        };
        Agent {
            cache,
            completion_model,
            listeners: vec![],
        }
    }

    pub fn insert_listener(&mut self, listener: impl AgentListener) {
        self.listeners.push(Box::new(listener));
    }

    pub async fn do_action<'a, F, Args, Fut, R>(
        &'a mut self,
        f: F,
        args: Args,
        trigger: Option<impl Into<ListenerTrigger>>,
    ) -> AgentResult<R>
    where
        F: for<'l> FnOnce(&'a mut Agent, Args) -> Fut,
        Fut: Future<Output = AgentResult<R>>,
    {
        if let Some(trigger) = trigger {
            self.use_listeners_with_trigger(trigger.into()).await?;
        }
        match f(self, args).await {
            Ok(result) => Ok(result),
            Err(err) => {
                warn!("error in do_action: {:?}", err);
                Err(err)
            }
        }
    }

    async fn use_listeners_with_trigger(&mut self, trigger: ListenerTrigger) -> AgentResult<()> {
        let mut ls = Vec::new();

        let mut i = 0;
        while i < self.listeners.len() {
            if self.listeners[i].trigger() == trigger {
                ls.push(self.listeners.remove(i));
            }
            i += 1;
        }

        for l in ls.iter_mut() {
            match l.sync_method(self) {
                Ok(()) => {
                    continue;
                }
                Err(_) => {
                    warn!("sync method is not implemented on this listener")
                }
            }
            match l.async_method(self).await {
                Ok(()) => {
                    continue;
                }
                Err(_) => {
                    warn!("async method is not implemented on this listener")
                }
            }
        }

        self.listeners.append(&mut ls);

        Ok(())
    }
}
