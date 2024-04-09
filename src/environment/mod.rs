pub mod agent_handle;
pub mod dispatch;
pub mod env_handle;
pub mod errors;
pub mod notification_stack;

use anyhow::anyhow;
use env_handle::EnvHandle;

use agent_handle::AgentHandle;
use dispatch::*;
use std::{collections::HashMap, sync::Arc};

use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{
    agents::{independent::IndependentAgent, Agent},
    environment::notification_stack::NotificationStack,
    language_models::ModelProvider,
};
pub use errors::*;

#[derive(Debug)]
pub struct Environment {
    pub id: String,
    handle_data: Option<HandleRequiredData>,
    sender: EnvMessageSender,
}

/// When environment handle thread is running, this data is owned by the EnvHandle
#[derive(Debug)]
pub(self) struct HandleRequiredData {
    pub dispatch: Arc<RwLock<Dispatch>>,
    pub listeners: Arc<RwLock<Vec<Box<dyn EnvListener>>>>,
}

impl Environment {
    /// New environment from id & api_keys, if id is None it will be a Uuid V4
    pub fn new(id: Option<&str>, api_keys: HashMap<ModelProvider, String>) -> Self {
        let id = match id {
            Some(id) => id.to_string(),
            None => Uuid::new_v4().to_string(),
        };

        let (s, r) = tokio::sync::mpsc::channel(1000);
        let sender = Arc::new(Mutex::new(s));
        let receiver = Arc::new(Mutex::new(r));
        let channel = EnvChannel::from((Arc::clone(&sender), receiver));

        let dispatch = Dispatch::new(channel, api_keys);
        let dispatch = Arc::new(RwLock::new(dispatch));
        let listeners = Arc::new(RwLock::new(vec![]));

        let handle_data = Some(HandleRequiredData {
            dispatch,
            listeners,
        });

        Self {
            id,
            sender,
            handle_data,
        }
    }

    /// Wraps method by the same name in inner Dispatch
    pub async fn make_agent_independent(&self, agent: Agent) -> Result<IndependentAgent, EnvError> {
        let dispatch = match &self.handle_data {
            None => return Err(EnvError::MissingHandleData),
            Some(d) => &d.dispatch,
        };
        Ok(dispatch.read().await.make_agent_independent(agent).await?)
    }

    /// Spawns env thread handle
    /// This will build a new notification stack and take a write lock on the
    /// environment's dispatch and listeners
    /// After calling this method the environment is pretty much inaccesible until
    /// the returned handle is finished
    #[tracing::instrument(name = "Spawn environment thread", skip(self))]
    pub fn spawn_handle(&mut self) -> Result<EnvHandle, EnvHandleError> {
        let mut handle = EnvHandle::from_env(self)?;
        handle.spawn()?;
        Ok(handle)
    }

    /// Finalize handle and return data to env
    pub async fn finalize(
        &mut self,
        handle: &mut EnvHandle,
    ) -> Result<NotificationStack, EnvError> {
        let stack = handle.finish_current_job().await.map_err(|err| {
            EnvError::Undefined(anyhow!("Error finishing thread in EnvHandle: {:?}", err))
        })?;
        let handle_data = handle
            .handle_data
            .take()
            .ok_or(EnvError::MissingHandleData)?;
        self.handle_data = Some(handle_data);
        Ok(stack)
    }

    /// Insert any struct implementing `EnvListener` trait
    pub async fn insert_listener(&mut self, listener: impl EnvListener) -> Result<(), EnvError> {
        match &self.handle_data {
            None => return Err(EnvError::MissingHandleData),
            Some(d) => d.listeners.write().await.push(Box::new(listener)),
        }
        Ok(())
    }

    /// Inserts agent into dispatch agent hashmap, returning a handle to the agent
    #[tracing::instrument(name = "Insert agent into dispatch")]
    pub async fn insert_agent(
        &mut self,
        id: Option<&str>,
        agent: Agent,
    ) -> Result<AgentHandle, EnvError> {
        let mut dispatch = match &self.handle_data {
            None => return Err(EnvError::MissingHandleData),
            Some(d) => d.dispatch.write().await,
        };

        let id = match id {
            Some(id) => id.to_string(),
            None => Uuid::new_v4().to_string(),
        };
        dispatch.agents.insert(id.clone(), agent);
        let handle = dispatch.get_agent_handle(&id).await.unwrap();
        drop(dispatch);
        Ok(handle)
    }

    /// Helper method for getting Arc clone of message sender
    pub(crate) fn clone_sender(&self) -> EnvMessageSender {
        Arc::clone(&self.sender)
    }
}
