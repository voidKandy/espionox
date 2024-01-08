pub mod agent;
pub mod dispatch;
pub mod errors;

use crate::Agent;
use anyhow::anyhow;
use dispatch::*;
use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{
    sync::{Mutex, RwLock, RwLockWriteGuard},
    task::JoinHandle,
};
use uuid::Uuid;

use self::{
    agent::{memory::Message, AgentHandle},
    errors::EnvError,
};

#[derive(Debug)]
pub struct EnvThreadHandle(JoinHandle<Result<(), EnvError>>);

#[derive(Debug)]
pub struct Environment {
    pub id: String,
    pub dispatch: Arc<RwLock<Dispatch>>,
    sender: EnvMessageSender,
    handle: Option<EnvThreadHandle>,
}

impl EnvThreadHandle {
    /// Join and resolve the current thread
    /// Env will need to be 'rerun' after calling this method
    pub async fn join(self) -> Result<(), EnvError> {
        self.0.await??;
        Ok(())
    }

    #[tracing::instrument(name = "Spawn EnvThreadHandle")]
    async fn spawn_thread(dispatch: Arc<RwLock<Dispatch>>) -> Result<Self, EnvError> {
        let handle: JoinHandle<Result<(), EnvError>> = tokio::spawn(async move {
            tracing::info!("Inside handle");
            let dispatch =
                tokio::time::timeout(std::time::Duration::from_millis(300), dispatch.write())
                    .await?;
            tracing::info!("Dispatch state: {:?}", dispatch);
            EnvThreadHandle::main_loop(dispatch).await
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(Self(handle))
    }

    #[tracing::instrument(name = "Dispatch main loop", skip(dispatch))]
    pub async fn main_loop(mut dispatch: RwLockWriteGuard<'_, Dispatch>) -> Result<(), EnvError> {
        let receiver: EnvMessageReceiver = Arc::clone(&dispatch.channel.receiver);
        loop {
            if let Some(message) = receiver
                .try_lock()
                .expect("Failed to lock recvr")
                .recv()
                .await
            {
                match message {
                    EnvMessage::Request(req) => {
                        tracing::info!("Dispatch received request: {:?}", req);
                        dispatch.handle_request(req).await?;
                    }
                    EnvMessage::Response(noti) => {
                        tracing::info!("Dispatch received notification: {:?}", noti);
                        dispatch.push_to_notifications(noti);
                    }
                    EnvMessage::Finish => break,
                }
            }
        }
        Ok(())
    }
}

impl Environment {
    pub fn clone_sender(&self) -> EnvMessageSender {
        Arc::clone(&self.sender)
    }

    pub async fn take_notifications(&mut self) -> Result<NotificationStack, EnvError> {
        let mut dispatch =
            tokio::time::timeout(std::time::Duration::from_millis(500), self.dispatch.write())
                .await?;
        dispatch
            .notifications
            .take()
            .ok_or(EnvError::Undefined(anyhow!("No notifications")))
    }

    /// Waits for a single notification with given ticket number to appear on dispatch stack and returns it
    #[tracing::instrument(name = "Wait for notification")]
    pub async fn wait_for_notification(&self, ticket: &Uuid) -> Result<EnvNotification, EnvError> {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                let dispatch_read = tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    self.dispatch.read(),
                )
                .await?;
                tracing::info!("Got dispatch read lock");
                if dispatch_read.notifications.is_none() {
                    tracing::info!("No notifications in dispatch, waiting 500ms");
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    drop(dispatch_read);
                } else {
                    break;
                }
            }
            loop {
                tracing::info!("Dispatch has notifications acquiring write lock");
                let mut dispatch_write = tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    self.dispatch.write(),
                )
                .await?;
                tracing::info!("Got dispatch write lock");
                let notis = dispatch_write.notifications.as_mut().unwrap();
                tracing::info!("Notification stack: {:?}", notis);
                if let Some(found_noti) = notis.take_by_ticket(*ticket) {
                    return Ok(found_noti);
                }
            }
        })
        .await?
    }

    /// Send finish request to dispatch and join thread handle
    #[tracing::instrument(name = "Send Finish message to dispatch", skip(self))]
    pub async fn finalize_dispatch(&mut self) -> Result<(), EnvError> {
        self.sender
            .lock()
            .await
            .send(EnvRequest::Finish.into())
            .await
            .map_err(|_| EnvError::Send)?;
        self.handle
            .take()
            .expect("Tried to finalize dispatch without an active handle")
            .join()
            .await?;
        Ok(())
    }

    /// Spawns env thread handle and waits until thread is ready
    #[tracing::instrument(name = "Spawn environment thread", skip(self))]
    pub async fn spawn(&mut self) -> Result<(), EnvError> {
        let dispatch_clone = Arc::clone(&self.dispatch);
        let handle = EnvThreadHandle::spawn_thread(dispatch_clone).await?;
        tracing::info!("Handle spawned, adding to environment");
        self.handle = Some(handle);
        Ok(())
    }

    /// Using the ID of an agent, get a it's handle
    pub async fn get_agent_handle(&self, id: &str) -> Option<AgentHandle> {
        let dispatch = self.dispatch.read().await;
        if let Some(_) = dispatch.agents.get(id) {
            let sender = Arc::clone(&dispatch.channel.sender);
            let handle = AgentHandle::from((id, sender));
            return Some(handle);
        }
        None
    }

    /// Inserts agent into dispatch agent hashmap, returning a handle to the agent
    #[tracing::instrument(name = "Insert agent into dispatch")]
    pub async fn insert_agent(
        &mut self,
        id: Option<&str>,
        agent: Agent,
    ) -> Result<AgentHandle, EnvError> {
        let mut dispatch = self.dispatch.write().await;
        let id = match id {
            Some(id) => id.to_string(),
            None => uuid::Uuid::new_v4().to_string(),
        };
        dispatch.agents.insert(id.clone(), agent);
        drop(dispatch);
        let handle = self.get_agent_handle(&id).await.unwrap();
        Ok(handle)
    }

    /// New environment from id & api_key, if id is None it will be a Uuid V4
    pub fn new(id: Option<&str>, api_key: Option<&str>) -> Self {
        let id = match id {
            Some(id) => id.to_string(),
            None => Uuid::new_v4().to_string(),
        };

        let (s, r) = tokio::sync::mpsc::channel(1000);
        let sender = Arc::new(Mutex::new(s));
        let receiver = Arc::new(Mutex::new(r));
        let channel = EnvChannel::from((Arc::clone(&sender), receiver));

        let dispatch = Dispatch::new(channel, api_key.map(|k| k.to_string()));
        let dispatch = Arc::new(RwLock::new(dispatch));

        Self {
            id,
            sender,
            dispatch,
            handle: None,
        }
    }
}
