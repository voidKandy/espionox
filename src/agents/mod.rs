pub mod actions;
pub mod error;
pub mod listeners;
pub mod memory;
use std::{fmt::Debug, future::Future};

use memory::MessageStack;
use tracing::warn;

use crate::language_models::{ModelProvider, LLM};
pub use error::AgentError;

use self::{
    error::AgentResult,
    listeners::{AgentListener, ListenerTrigger},
};

/// Agent struct for interracting with LLM
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub cache: MessageStack,
    pub(crate) completion_handler: LLM,
    #[serde(skip)]
    listeners: Vec<Box<dyn listeners::AgentListener>>,
}

impl Agent {
    /// For creating an Agent given optional system prompt content and model
    pub fn new(init_prompt: Option<&str>, completion_handler: LLM) -> Self {
        let cache = match init_prompt {
            Some(p) => MessageStack::new(p),
            None => MessageStack::init(),
        };
        Agent {
            cache,
            completion_handler,
            listeners: vec![],
        }
    }

    pub fn provider(&self) -> ModelProvider {
        self.completion_handler.provider()
    }

    pub fn insert_listener(&mut self, listener: impl AgentListener) {
        self.listeners.push(Box::new(listener));
    }

    pub async fn use_listeners_with_trigger(
        &mut self,
        trigger: ListenerTrigger,
    ) -> AgentResult<()> {
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
                    warn!("Sync method is not implemented on this listener")
                }
            }
            match l.async_method(self).await {
                Ok(()) => {
                    continue;
                }
                Err(_) => {
                    warn!("Async method is not implemented on this listener")
                }
            }
        }

        self.listeners.append(&mut ls);

        Ok(())
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
        f(self, args).await
    }
}
