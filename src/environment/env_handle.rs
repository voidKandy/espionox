use std::{collections::VecDeque, sync::Arc, time::Duration};
use tokio::{
    sync::{RwLock, RwLockWriteGuard},
    task::JoinHandle,
};
use uuid::Uuid;

use super::{errors::*, Dispatch, EnvListener, EnvMessage, EnvMessageReceiver, EnvMessageSender};
use super::{notification_stack::NotificationStack, EnvRequest};
use super::{notification_stack::RefCountedNotificationStack, HandleRequiredData};
use super::{EnvNotification, Environment};

use super::agent_handle::EndpointCompletionHandler;

/// Takes write ownership of dispatch when the Environment is spawned
#[derive(Debug)]
pub struct EnvHandle<H>
where
    H: EndpointCompletionHandler,
{
    pub notifications: Option<RefCountedNotificationStack>,
    pub(super) handle_data: Option<HandleRequiredData<H>>,
    thread_handle: Option<JoinHandle<Result<(), EnvError>>>,
    sender: EnvMessageSender,
}

impl<H> EnvHandle<H>
where
    H: EndpointCompletionHandler,
{
    /// Returns boolean if environment thread handle has already been spawned
    pub fn is_running(&self) -> bool {
        self.thread_handle.is_some()
    }

    /// Send finish request to dispatch and join thread handle
    #[tracing::instrument(name = "Send Finish message to dispatch", skip(self))]
    pub async fn finish_current_job(&mut self) -> Result<NotificationStack, EnvHandleError> {
        self.sender
            .lock()
            .await
            .send(EnvRequest::Finish.into())
            .await
            .map_err(|_| EnvError::Send)?;
        self.join().await?;
        self.thread_handle = None;
        match self.notifications.take() {
            Some(stack) => {
                if let Some(stack) = Arc::into_inner(stack) {
                    let stack = stack.into_inner();
                    return Ok(stack);
                }
                Err(EnvHandleError::CouldNotOwnNotifications)
            }
            None => Err(EnvHandleError::MissingNotifications),
        }
    }

    /// Restarts thread and returns message stack
    pub async fn restart(&mut self) -> Result<NotificationStack, EnvHandleError> {
        let stack = self.finish_current_job().await?;
        self.spawn()?;
        Ok(stack)
    }

    /// Waits for a single notification with given ticket number to appear on dispatch stack, removes it and returns it
    #[tracing::instrument(name = "Wait for notification", skip(self))]
    pub async fn wait_for_notification(
        &mut self,
        ticket: &Uuid,
    ) -> Result<EnvNotification, EnvHandleError> {
        let notis = self
            .notifications
            .as_mut()
            .ok_or(EnvHandleError::MissingNotifications)
            .unwrap();

        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let notis_read = notis.read().await;
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
                    tokio::time::timeout(std::time::Duration::from_millis(1000), notis.write())
                        .await
                        .map_err(|err| EnvHandleError::from(EnvError::from(err)))?;
                tracing::info!("Got notis write lock");
                if let Some(found_noti) = notis_write.take_by_ticket(*ticket) {
                    tracing::info!("Found matching notification: {:?}", found_noti);
                    return Ok(found_noti);
                }
            }
        })
        .await
        .map_err(|err| EnvHandleError::from(EnvError::from(err)))?
    }

    /// Returns another sender to the env receiver
    pub fn new_sender(&self) -> EnvMessageSender {
        Arc::clone(&self.sender)
    }

    /// Spawns an EnvHandle given a mutable reference to an Environment
    /// Doing this starts a new notification stack which is updated from within
    /// the thread_handle
    pub fn spawn(&mut self) -> Result<(), EnvHandleError> {
        if self.is_running() {
            return Err(EnvHandleError::ThreadAlreadySpawned);
        }
        let handle_data = match &self.handle_data {
            None => return Err(EnvHandleError::MissingHandleData),
            Some(d) => d,
        };
        self.notifications = Some(Arc::new(RwLock::new(VecDeque::new().into())));

        let d_clone = Arc::clone(&handle_data.dispatch);
        let l_clone = Arc::clone(&handle_data.listeners).into();
        let n_clone = Arc::clone(&self.notifications.as_ref().unwrap()).into();

        let thread_handle: JoinHandle<Result<(), EnvError>> = tokio::spawn(async move {
            tracing::info!("Inside handle");
            let dispatch =
                tokio::time::timeout(std::time::Duration::from_millis(300), d_clone.write())
                    .await?;
            tracing::info!("Dispatch state: {:?}", dispatch);
            Self::main_loop(dispatch, n_clone, l_clone).await
        });
        self.thread_handle = Some(thread_handle);
        Ok(())
    }

    /// Join and resolve the current thread
    /// In order to make more requests
    /// You will need to call `spawn` on this env again after calling this method
    async fn join(&mut self) -> Result<(), EnvHandleError> {
        match self.thread_handle.take() {
            Some(handle) => Ok(handle
                .await
                .map_err(|err| EnvHandleError::from(EnvError::from(err)))??),
            None => Err(EnvHandleError::MissingThreadHandle),
        }
    }

    pub(super) fn from_env(env: &mut Environment<H>) -> Result<Self, EnvHandleError> {
        let sender = env.clone_sender();
        let handle_data = env
            .handle_data
            .take()
            .ok_or(EnvHandleError::MissingHandleData)?;

        Ok(Self {
            notifications: None,
            handle_data: Some(handle_data),
            thread_handle: None,
            sender,
        })
    }

    #[tracing::instrument(name = "Dispatch main loop", skip_all)]
    async fn main_loop(
        mut dispatch: RwLockWriteGuard<'_, Dispatch<H>>,
        noti_stack: RefCountedNotificationStack,
        listeners: Arc<RwLock<Vec<Box<dyn EnvListener<H>>>>>,
    ) -> Result<(), EnvError> {
        let receiver: EnvMessageReceiver = Arc::clone(&dispatch.channel.receiver);
        loop {
            if let Some(message) = receiver
                .try_lock()
                .expect("Failed to lock recvr")
                .recv()
                .await
            {
                let message = super::dispatch::listeners::run_listeners(
                    message,
                    Arc::clone(&listeners),
                    &mut dispatch,
                )
                .await?;
                match message {
                    EnvMessage::Request(req) => {
                        tracing::info!("Dispatch received request: {:?}", req);
                        dispatch.requests.push_front(req);
                    }
                    EnvMessage::Response(noti) => {
                        tracing::info!("Dispatch received notification: {:?}", noti);
                        dispatch.handle_notification(&noti).await?;
                        noti_stack.write().await.push(noti).await;
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
