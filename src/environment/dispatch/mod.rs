mod channel;
pub mod listeners;
pub use channel::*;
pub use listeners::EnvListener;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::AgentHandle;
use reqwest::Client;
use std::{collections::VecDeque, sync::Arc};

use crate::agents::{
    independent::IndependentAgent,
    language_models::openai::gpt::streaming::{
        CompletionStreamReceiver, CompletionStreamSender, StreamedCompletionHandler,
    },
    memory::{Message, MessageRole, MessageStack},
    Agent,
};
use std::collections::HashMap;

use super::errors::DispatchError;

pub type AgentHashMap = HashMap<String, Agent>;

#[derive(Debug)]
pub struct Dispatch {
    api_key: Option<String>,
    pub client: Client,
    pub(super) requests: VecDeque<EnvRequest>,
    // pub(super) listeners: Vec<Box<dyn EnvListener>>,
    pub(super) channel: EnvChannel,
    pub(super) agents: AgentHashMap,
}

impl Dispatch {
    /// Using the api key and client already in dispatch, make an agent independent
    pub async fn make_agent_independent(
        &self,
        agent: Agent,
    ) -> Result<IndependentAgent, DispatchError> {
        let api_key = self.api_key()?;
        let client = self.client.clone();
        Ok(IndependentAgent::new(agent, client, api_key))
    }
    /// Get a mutable reference to an agent within the dispatch
    pub fn get_agent_mut(&mut self, id: &str) -> Result<&mut Agent, DispatchError> {
        if let Some(agent) = self.agents.get_mut(id) {
            return Ok(agent);
        }
        Err(DispatchError::AgentIsNone)
    }

    /// Get a immutable reference to an agent within the dispatch
    pub fn get_agent_ref(&self, id: &str) -> Result<&Agent, DispatchError> {
        if let Some(agent) = self.agents.get(id) {
            return Ok(agent);
        }
        Err(DispatchError::AgentIsNone)
    }

    /// Get the api key of the dispatch
    /// TODO!
    /// THIS METHOD WILL NEED TO CHANGE WHEN MORE MODELS ARE SUPPORTED
    pub fn api_key(&self) -> Result<String, DispatchError> {
        self.api_key.clone().ok_or(DispatchError::NoApiKey)
    }

    /// Using the ID of an agent, get it's handle
    pub async fn get_agent_handle(&self, id: &str) -> Result<AgentHandle, DispatchError> {
        if let Some(_) = self.agents.get(id) {
            let sender = Arc::clone(&self.channel.sender);
            let handle = AgentHandle::from((id, sender));
            return Ok(handle);
        }
        Err(DispatchError::AgentIsNone)
    }

    pub(crate) fn new(channel: EnvChannel, api_key: Option<String>) -> Self {
        Self {
            api_key,
            requests: VecDeque::new(),
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
        let ticket = uuid::Uuid::new_v4();
        sender
            .lock()
            .await
            .send(
                EnvNotification::AgentStateUpdate {
                    ticket,
                    agent_id,
                    cache,
                }
                .into(),
            )
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
                let message = Message::new_assistant(&json.to_string());
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

            EnvRequest::GetAgentState { ticket, agent_id } => {
                let agent = self.get_agent_ref(&agent_id)?;
                let cache = agent.cache.clone();
                let sender = &self.channel.sender.lock().await;
                let response = EnvNotification::AgentStateUpdate {
                    ticket,
                    agent_id,
                    cache,
                };
                sender
                    .send(response.into())
                    .await
                    .map_err(|_| DispatchError::Send)
            }

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
                        agent.cache.mut_filter_by(MessageRole::System, true);
                    }
                    false => {
                        agent.cache = MessageStack::init();
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
                let message = Message::new_assistant(&res_str);

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
