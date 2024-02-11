pub mod agent;
pub mod dispatch;
pub mod errors;

use crate::Agent;
use dispatch::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};
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
pub struct NotificationStack(pub Arc<RwLock<VecDeque<EnvNotification>>>);

impl Clone for NotificationStack {
    fn clone(&self) -> Self {
        Arc::clone(&self.0).into()
    }
}

impl From<Arc<RwLock<VecDeque<EnvNotification>>>> for NotificationStack {
    fn from(value: Arc<RwLock<VecDeque<EnvNotification>>>) -> Self {
        Self(value)
    }
}

impl Into<Arc<RwLock<VecDeque<EnvNotification>>>> for NotificationStack {
    fn into(self) -> Arc<RwLock<VecDeque<EnvNotification>>> {
        self.0
    }
}
#[derive(Debug)]
pub struct EnvThreadHandle(JoinHandle<Result<(), EnvError>>);

#[derive(Debug)]
pub struct Environment {
    pub id: String,
    pub dispatch: Arc<RwLock<Dispatch>>,
    pub notifications: NotificationStack,
    // maybe a good idea?
    // pub agent_handles: HashMap<String, AgentHandle>,
    listeners: Arc<RwLock<Vec<Box<dyn EnvListener>>>>,
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

    #[tracing::instrument(name = "Dispatch main loop", skip_all)]
    pub async fn main_loop(
        mut dispatch: RwLockWriteGuard<'_, Dispatch>,
        noti_stack: NotificationStack,
        listeners: Arc<RwLock<Vec<Box<dyn EnvListener>>>>,
    ) -> Result<(), EnvError> {
        let receiver: EnvMessageReceiver = Arc::clone(&dispatch.channel.receiver);
        loop {
            if let Some(message) = receiver
                .try_lock()
                .expect("Failed to lock recvr")
                .recv()
                .await
            {
                let message =
                    dispatch::run_listeners(message, Arc::clone(&listeners), &mut dispatch).await?;
                match message {
                    EnvMessage::Request(req) => {
                        tracing::info!("Dispatch received request: {:?}", req);
                        dispatch.requests.push_front(req);
                    }
                    EnvMessage::Response(noti) => {
                        tracing::info!("Dispatch received notification: {:?}", noti);
                        dispatch.handle_notification(&noti).await?;
                        noti_stack.push(noti).await;
                    }
                    EnvMessage::Finish => break,
                }
            }
            if let Some(req) = dispatch.requests.pop_back() {
                if req == EnvRequest::Finish && dispatch.requests.len() > 0 {
                    tracing::info!(
                        "Finish request in stack, but stack has requests. Pushing it back"
                    );
                    dispatch.requests.push_front(req)
                } else {
                    dispatch.handle_request(req).await?;
                }
            }
        }
        Ok(())
    }
}

impl Environment {
    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn clone_sender(&self) -> EnvMessageSender {
        Arc::clone(&self.sender)
    }

    /// Spawns env thread handle and waits until thread is ready
    #[tracing::instrument(name = "Spawn environment thread", skip(self))]
    pub async fn spawn(&mut self) -> Result<(), EnvError> {
        let dispatch_clone = Arc::clone(&self.dispatch);
        let noti_stack_clone = Arc::clone(&self.notifications.0).into();
        let listeners_clone = Arc::clone(&self.listeners).into();

        let handle: JoinHandle<Result<(), EnvError>> = tokio::spawn(async move {
            tracing::info!("Inside handle");
            let dispatch = tokio::time::timeout(
                std::time::Duration::from_millis(300),
                dispatch_clone.write(),
            )
            .await?;
            tracing::info!("Dispatch state: {:?}", dispatch);
            EnvThreadHandle::main_loop(dispatch, noti_stack_clone, listeners_clone).await
        });

        tracing::info!("Handle spawned, adding to environment");
        self.handle = Some(EnvThreadHandle(handle));
        Ok(())
    }

    pub async fn insert_listener(&mut self, listener: impl EnvListener) {
        self.listeners.write().await.push(Box::new(listener))
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
        let handle = dispatch.get_agent_handle(&id).await.unwrap();
        drop(dispatch);
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

        let notifications = Arc::new(RwLock::new(VecDeque::new())).into();
        let listeners = Arc::new(RwLock::new(vec![]));

        Self {
            id,
            sender,
            dispatch,
            listeners,
            notifications,
            handle: None,
        }
    }
}

impl NotificationStack {
    /// Pushes given notification to the front
    pub async fn push(&self, noti: EnvNotification) {
        let mut write = self.0.write().await;
        match noti {
            EnvNotification::AgentStateUpdate { ref agent_id, .. } => {
                let outer_id = agent_id;
                write.retain(|noti| match noti {
                    EnvNotification::AgentStateUpdate { agent_id, .. } => &agent_id != &outer_id,
                    _ => true,
                });
            }
            _ => {}
        }
        write.push_front(noti);
    }

    /// Removes notifications with given agent id from stack and returns them as VecDeque
    pub fn take_by_agent(
        vec: &mut VecDeque<EnvNotification>,
        agent_id: &str,
    ) -> Option<VecDeque<EnvNotification>> {
        let (matching, remaining): (VecDeque<EnvNotification>, VecDeque<EnvNotification>) = vec
            .drain(..)
            .partition(|noti| noti.agent_id() == Some(agent_id));
        *vec = remaining;
        if matching.len() != 0 {
            return Some(matching.into());
        }
        None
    }

    /// Removes the notification with the given ticket number from the stack
    pub fn take_by_ticket(
        vec: &mut VecDeque<EnvNotification>,
        ticket: Uuid,
    ) -> Option<EnvNotification> {
        if let Some(index) = vec
            .iter_mut()
            .position(|noti| noti.ticket_number() == Some(ticket))
        {
            vec.remove(index)
        } else {
            None
        }
    }

    /// Waits for a single notification with given ticket number to appear on dispatch stack, removes it and returns it
    #[tracing::instrument(name = "Wait for notification", skip(self))]
    pub async fn wait_for_notification(&self, ticket: &Uuid) -> Result<EnvNotification, EnvError> {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let notis_read = self.0.read().await;
                tracing::info!("Got notis read lock");
                if notis_read.len() == 0 {
                    tracing::info!("No notifications, waiting");
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    drop(notis_read);
                } else {
                    break;
                }
            }
            loop {
                tracing::info!("Notifications is Some, acquiring write lock");
                let mut notis_write =
                    tokio::time::timeout(std::time::Duration::from_millis(1000), self.0.write())
                        .await?;
                tracing::info!("Got notis write lock");
                if let Some(found_noti) = Self::take_by_ticket(&mut notis_write, *ticket) {
                    tracing::info!("Found matching notification: {:?}", found_noti);
                    return Ok(found_noti);
                }
            }
        })
        .await?
    }
}
