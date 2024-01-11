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
    errors::{DispatchError, EnvError}
};

#[derive(Debug)]
pub struct NotificationStack(pub VecDeque<EnvNotification>);

impl From<VecDeque<EnvNotification>> for NotificationStack {
    fn from(value: VecDeque<EnvNotification>) -> Self {
        Self(value)
    }
}

impl Into<VecDeque<EnvNotification>> for NotificationStack {
    fn into(self) -> VecDeque<EnvNotification> {
        self.0
    }
}

#[derive(Debug)]
pub struct EnvThreadHandle(JoinHandle<Result<(), EnvError>>);

#[derive(Debug)]
pub struct Environment {
    pub id: String,
    pub dispatch: Arc<RwLock<Dispatch>>,
    pub notifications: Arc<RwLock<Option<NotificationStack>>>,
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

    #[tracing::instrument(name = "Push notification to stack")]
    pub(crate) async fn push_to_notifications(noti_stack: Arc<RwLock<Option<NotificationStack>>>, noti: EnvNotification)  {
        tracing::info!("Pushing {:?} to noti stack", noti);
        let mut noti_write = noti_stack.write().await ;
        noti_write.get_or_insert_with(|| VecDeque::new().into()).0.push_front(noti);
        tracing::info!("Stack after push: {:?}", noti_write);
    }

    #[tracing::instrument(name = "Dispatch main loop", skip(dispatch))]
    pub async fn main_loop(mut dispatch: RwLockWriteGuard<'_, Dispatch>, noti_stack: Arc<RwLock<Option<NotificationStack>>>) -> Result<(), EnvError> {
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
                        dispatch.handle_notification(&noti).await?;
                        Self::push_to_notifications(Arc::clone(&noti_stack),noti).await;
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


    /// Spawns env thread handle and waits until thread is ready
    #[tracing::instrument(name = "Spawn environment thread", skip(self))]
    pub async fn spawn(&mut self) -> Result<(), EnvError> {
        let dispatch_clone = Arc::clone(&self.dispatch);
        let noti_stack_clone = Arc::clone(&self.notifications);

        let handle: JoinHandle<Result<(), EnvError>> = tokio::spawn(async move {
            tracing::info!("Inside handle");
            let dispatch =
                tokio::time::timeout(std::time::Duration::from_millis(300), dispatch_clone.write())
                    .await?;
            tracing::info!("Dispatch state: {:?}", dispatch);
            EnvThreadHandle::main_loop(dispatch, noti_stack_clone).await
        });

        tracing::info!("Handle spawned, adding to environment");
        self.handle = Some(EnvThreadHandle(handle));
        Ok(())
    }

    pub async fn take_notifications(&mut self) -> Result<NotificationStack, EnvError> {
           self 
            .notifications.write().await
            .take()
            .ok_or(EnvError::Undefined(anyhow!("No notifications")))
    }

    /// Waits for a single notification with given ticket number to appear on dispatch stack, removes it and returns it
    #[tracing::instrument(name = "Wait for notification")]
    pub async fn wait_for_notification(&self, ticket: &Uuid) -> Result<EnvNotification, EnvError> {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                let notis_read = 
                    // tokio::time::timeout(
                    // std::time::Duration::from_millis(1000),
                    self.notifications.read()
                    // ,
                // )
                .await;
                tracing::info!("Got notis read lock");
                if notis_read.is_none() {
                    tracing::info!("No notifications, waiting");
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    drop(notis_read);
                } else {
                    break;
                }
            }
            loop {
                tracing::info!("Notifications is Some, acquiring write lock");
                let mut notis_write = tokio::time::timeout(
                    std::time::Duration::from_millis(1000),
                    self.notifications.write(),
                )
                .await?;
                tracing::info!("Got notis write lock");
                let notis = notis_write.as_mut().unwrap();
                tracing::info!("Notification stack: {:?}", notis);
                if let Some(found_noti) = notis.take_by_ticket(*ticket) {
                    tracing::info!("Found matching notification: {:?}", found_noti);
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

        let notifications = Arc::new(RwLock::new(None));

        Self {
            id,
            sender,
            dispatch,
            notifications,
            handle: None,
        }
    }
}



impl NotificationStack {
    /// Removes notifications with given agent id from stack and returns them as VecDeque
    pub fn take_by_agent(&mut self, agent_id: &str) -> Option<VecDeque<EnvNotification>> {
        let (matching, remaining): (VecDeque<EnvNotification>, VecDeque<EnvNotification>) = self
            .0
            .drain(..)
            .partition(|noti| noti.agent_id() == Some(agent_id));
        *self = Self::from(remaining);
        if matching.len() != 0 {
            return Some(matching.into());
        }
        None
    }

    /// Removes the notification with the given ticket number from the stack
    pub fn take_by_ticket(&mut self, ticket: Uuid) -> Option<EnvNotification> {
        if let Some(index) = self
            .0
            .iter_mut()
            .position(|noti| noti.ticket_number() == Some(ticket))
        {
            self.0.remove(index)
        } else {
            None
        }
    }
}
