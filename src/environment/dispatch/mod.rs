mod channel;
pub use channel::*;
use tokio::sync::Mutex;
use uuid::Uuid;

use reqwest::Client;
use std::{collections::VecDeque, sync::Arc};

use crate::{
    environment::agent::{
        language_models::openai::gpt::streaming_utils::*,
        memory::{messages::MessageRole, Message, MessageVector},
    },
    Agent,
};
use std::collections::HashMap;

use super::errors::DispatchError;

pub type AgentHashMap = HashMap<String, Agent>;

#[derive(Debug)]
pub struct Dispatch {
    api_key: Option<String>,
    pub(super) channel: EnvChannel,
    pub(super) client: Client,
    pub(super) agents: AgentHashMap,
    // pub(super) notifications: Option<NotificationStack>,
}

impl Dispatch {
    pub fn get_agent_mut(&mut self, id: &str) -> Result<&mut Agent, DispatchError> {
        if let Some(agent) = self.agents.get_mut(id) {
            return Ok(agent);
        }
        Err(DispatchError::AgentIsNone)
    }

    pub fn get_agent_ref(&self, id: &str) -> Result<&Agent, DispatchError> {
        if let Some(agent) = self.agents.get(id) {
            return Ok(agent);
        }
        Err(DispatchError::AgentIsNone)
    }

    pub fn api_key(&self) -> Result<String, DispatchError> {
        self.api_key.clone().ok_or(DispatchError::NoApiKey)
    }

    pub(crate) fn new(channel: EnvChannel, api_key: Option<String>) -> Self {
        Self {
            api_key,
            channel,
            client: Client::new(),
            agents: HashMap::new(),
        }
    }

    #[tracing::instrument(name = "Push message to agent cache")]
    async fn push_to_agent_cache(
        agent: &mut Agent,
        agent_id: &str,
        message: &Message,
        sender: &EnvMessageSender,
    ) -> Result<(), DispatchError> {
        agent.cache.push(message.to_owned());
        let cache = agent.cache.clone();
        let agent_id = agent_id.to_string();
        sender
            .lock()
            .await
            .send(EnvNotification::CacheUpdate { agent_id, cache }.into())
            .await
            .map_err(|_| DispatchError::Send)?;
        Ok(())
    }

    pub(super) async fn handle_notification(
        &mut self,
        noti: &EnvNotification,
    ) -> Result<(), DispatchError> {
        match noti {
            EnvNotification::GotCompletionResponse {
                agent_id, message, ..
            } => self
                .channel
                .sender
                .lock()
                .await
                .send(
                    EnvRequest::PushToCache {
                        agent_id: agent_id.clone(),
                        message: message.clone(),
                    }
                    .into(),
                )
                .await
                .map_err(|_| DispatchError::Send),
            EnvNotification::GotFunctionResponse { agent_id, json, .. } => {
                let message = Message::new(MessageRole::Assistant, &json.to_string());
                self.channel
                    .sender
                    .lock()
                    .await
                    .send(
                        EnvRequest::PushToCache {
                            agent_id: agent_id.clone(),
                            message,
                        }
                        .into(),
                    )
                    .await
                    .map_err(|_| DispatchError::Send)?;
                Ok(())
            }

            _ => Ok(()),
        }
    }

    #[tracing::instrument(name = "Handle dispatch request")]
    pub(super) async fn handle_request(&mut self, req: EnvRequest) -> Result<(), DispatchError> {
        let response = match req {
            EnvRequest::Finish => self.finish().await,

            EnvRequest::PushToCache { agent_id, message } => {
                let sender = Arc::clone(&self.channel.sender);
                let agent = self.get_agent_mut(&agent_id)?;
                Self::push_to_agent_cache(agent, &agent_id, &message, &sender).await
            }

            EnvRequest::ResetCache {
                agent_id,
                keep_sys_message,
            } => {
                let agent = self.get_agent_mut(&agent_id)?;
                match keep_sys_message {
                    true => {
                        agent.cache.reset_to_system_prompt();
                    }
                    false => {
                        agent.cache = MessageVector::init();
                    }
                }
                Ok(())
            }

            EnvRequest::GetCompletion { ticket, agent_id } => {
                let api_key = self.api_key()?;
                let client = &self.client;
                let agent = self.get_agent_ref(&agent_id)?;

                let completion_fn = agent.model.io_completion_fn();
                let payload = &(&agent.cache).into();
                let response = completion_fn(&client, &api_key, payload, &agent.model).await?;

                let agent = self.get_agent_mut(&agent_id)?;
                let res_str = agent.handle_completion_response(response)?;
                let message = Message::new(MessageRole::Assistant, &res_str);

                self.channel
                    .sender
                    .lock()
                    .await
                    .send(
                        EnvNotification::GotCompletionResponse {
                            ticket,
                            agent_id,
                            message,
                        }
                        .into(),
                    )
                    .await
                    .map_err(|_| DispatchError::Send)?;

                Ok(())
            }

            EnvRequest::GetFunctionCompletion {
                ticket,
                agent_id,
                function,
            } => {
                let client = &self.client;
                let api_key = self.api_key()?;
                let agent = self.get_agent_ref(&agent_id)?;

                let completion_fn = agent.model.function_completion_fn();
                let payload = &(&agent.cache).into();
                let response =
                    completion_fn(&client, &api_key, payload, &agent.model, &function).await?;

                let json = response.parse_fn()?;

                self.channel
                    .sender
                    .lock()
                    .await
                    .send(
                        EnvNotification::GotFunctionResponse {
                            ticket,
                            agent_id: agent_id.clone(),
                            json,
                        }
                        .into(),
                    )
                    .await
                    .map_err(|_| DispatchError::Send)?;
                Ok(())
            }

            EnvRequest::GetCompletionStreamHandle { ticket, agent_id } => {
                let api_key = self.api_key()?;
                let client = &self.client;
                let agent = self.get_agent_ref(&agent_id)?;

                let completion_fn = agent.model.stream_completion_fn();
                let payload = &(&agent.cache).into();
                let response = completion_fn(&client, &api_key, payload, &agent.model).await?;

                let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
                    tokio::sync::mpsc::channel(50);
                let handler = Arc::new(Mutex::new(StreamedCompletionHandler::from((
                    response, tx, rx,
                ))));

                self.channel
                    .sender
                    .lock()
                    .await
                    .send(
                        EnvNotification::GotStreamHandle {
                            ticket,
                            agent_id,
                            handler,
                        }
                        .into(),
                    )
                    .await
                    .map_err(|_| DispatchError::Send)?;
                Ok(())
            }
        };
        tracing::info!("Got response from dispatch request: {:?}", response);
        Ok(response?)
    }

    async fn finish(&mut self) -> Result<(), DispatchError> {
        let sender_clone: EnvMessageSender = Arc::clone(&self.channel.sender);
        sender_clone
            .lock()
            .await
            .send(EnvMessage::Finish)
            .await
            .map_err(|_| DispatchError::Send)?;
        Ok(())
    }
}
