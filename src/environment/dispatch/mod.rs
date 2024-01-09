mod channel;
pub use channel::*;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::Agent;
use reqwest::Client;
use std::{collections::VecDeque, sync::Arc};

use crate::environment::agent::{
    language_models::openai::gpt::streaming_utils::*,
    memory::{messages::MessageRole, Message},
};
use std::collections::HashMap;

use super::{agent::language_models::openai::functions::Function, errors::DispatchError};

pub type AgentHashMap = HashMap<String, Agent>;

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
pub struct Dispatch {
    pub(super) api_key: Option<String>,
    pub(super) channel: EnvChannel,
    pub(super) agents: AgentHashMap,
    // pub(super) notifications: Option<NotificationStack>,
}

impl Dispatch {
    pub fn get_agent_by_id(&mut self, id: &str) -> Result<&mut Agent, DispatchError> {
        if let Some(agent) = self.agents.get_mut(id) {
            return Ok(agent);
        }
        Err(DispatchError::AgentIsNone)
    }

    pub(crate) fn new(channel: EnvChannel, api_key: Option<String>) -> Self {
        Self {
            api_key,
            channel,
            agents: HashMap::new(),
            // notifications: None,
        }
    }

    #[tracing::instrument(name = "update agent cache")]
    async fn push_to_agent_cache(
        agent: &mut Agent,
        agent_id: &str,
        message: &Message,
        sender: &EnvMessageSender,
    ) -> Result<(), DispatchError> {
        // let agent_id = &agent.id;
        agent.cache.push(message.clone());
        sender
            .try_lock()
            .expect("Failed to lock sender")
            .send(
                EnvNotification::ChangedCache {
                    agent_id: agent_id.to_string(),
                    message: message.clone(),
                }
                .into(),
            )
            .await
            .map_err(|_| DispatchError::Send)?;
        Ok(())
    }

    #[tracing::instrument(name = "Handle dispatch request")]
    pub(super) async fn handle_request(&mut self, req: EnvRequest) -> Result<(), DispatchError> {
        let response = match req {
            EnvRequest::Finish => self.finish().await,
            EnvRequest::PromptAgent {
                agent_id,
                message,
                ticket,
            } => self.prompt_agent(ticket, &agent_id, message).await,
            EnvRequest::StreamPromptAgent {
                agent_id,
                message,
                ticket,
            } => self.stream_prompt_agent(ticket, &agent_id, message).await,
            EnvRequest::FunctionPromptAgent {
                ticket,
                agent_id,
                function,
                message,
            } => {
                self.function_prompt_agent(ticket, &agent_id, message, function)
                    .await
            }

            EnvRequest::UpdateCache { agent_id, message } => {
                let sender = Arc::clone(&self.channel.sender);
                let agent = self.get_agent_by_id(&agent_id)?;
                Self::push_to_agent_cache(agent, &agent_id, &message, &sender).await
            }
        };
        tracing::info!("Got response from dispatch request: {:?}", response);
        Ok(response?)
    }

    #[tracing::instrument(name = "Push response to dispatch stack")]
    fn handle_response(&mut self, res: EnvNotification) {}

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

    async fn prompt_agent(
        &mut self,
        ticket: Uuid,
        agent_id: &str,
        message: Message,
    ) -> Result<(), DispatchError> {
        let sender_clone: EnvMessageSender = Arc::clone(&self.channel.sender);
        let api_key = &self.api_key.clone().ok_or(DispatchError::NoApiKey)?;
        let agent = self.get_agent_by_id(&agent_id)?;

        Self::push_to_agent_cache(agent, &agent_id, &message, &sender_clone)
            .await
            .expect("Failed to push to agent cache");

        let completion_fn = agent.model.io_completion_fn();
        let payload = &(&agent.cache).into();
        let client = Client::new();
        let response = completion_fn(&client, api_key, payload, &agent.model).await?;

        let res_str = agent.handle_completion_response(response)?;
        let message = Message::new(MessageRole::Assistant, &res_str);
        tracing::info!(
            "Got completion message in response: {:?}, Pushing to agent cache",
            message
        );
        Self::push_to_agent_cache(agent, &agent_id, &message, &sender_clone)
            .await
            .expect("Failed to push to agent cache");
        let assistant_message_response = EnvNotification::GotMessageResponse {
            ticket,
            agent_id: agent_id.to_string(),
            message,
        };
        self.channel
            .sender
            .lock()
            .await
            .send(assistant_message_response.into())
            .await
            .map_err(|_| DispatchError::Send)
    }

    #[tracing::instrument(name = "Prompt agent for a streamed response", skip(self))]
    async fn stream_prompt_agent(
        &mut self,
        ticket: Uuid,
        agent_id: &str,
        message: Message,
    ) -> Result<(), DispatchError> {
        let sender_clone: EnvMessageSender = Arc::clone(&self.channel.sender);
        let api_key = &self.api_key.clone().ok_or(DispatchError::NoApiKey)?;
        let agent = self.get_agent_by_id(&agent_id)?;

        Self::push_to_agent_cache(agent, &agent_id, &message, &sender_clone)
            .await
            .expect("Failed to push to agent cache");

        let completion_fn = agent.model.stream_completion_fn();
        let payload = &(&agent.cache).into();
        let client = Client::new();
        let response = completion_fn(&client, api_key, payload, &agent.model).await?;
        tracing::info!("Got response from stream completion function");

        let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
            tokio::sync::mpsc::channel(50);
        let handler = Arc::new(Mutex::new(StreamedCompletionHandler::from((
            response, tx, rx,
        ))));
        let notification = EnvNotification::GotStreamHandle {
            ticket,
            agent_id: agent_id.to_string(),
            handler,
        };
        tracing::info!("Got stream receiver, sending as notification");
        sender_clone
            .lock()
            .await
            .send(notification.into())
            .await
            .map_err(|_| DispatchError::Send)?;
        Ok(())
    }

    async fn function_prompt_agent(
        &mut self,
        ticket: Uuid,
        agent_id: &str,
        message: Message,
        function: Function,
    ) -> Result<(), DispatchError> {
        let sender_clone: EnvMessageSender = Arc::clone(&self.channel.sender);
        let api_key = &self.api_key.clone().ok_or(DispatchError::NoApiKey)?;
        let agent = self.get_agent_by_id(&agent_id)?;

        Self::push_to_agent_cache(agent, &agent_id, &message, &sender_clone)
            .await
            .expect("Failed to push to agent cache");

        let completion_fn = agent.model.function_completion_fn();
        let payload = &(&agent.cache).into();
        let client = Client::new();
        let response = completion_fn(&client, api_key, payload, &agent.model, &function).await?;
        let json = response.parse_fn()?;

        let message = Message::new(MessageRole::Assistant, &json.to_string());
        tracing::info!(
            "Got completion message in response: {:?}, Pushing to agent cache",
            message
        );
        Self::push_to_agent_cache(agent, &agent_id, &message, &sender_clone)
            .await
            .expect("Failed to push to agent cache");
        let assistant_message_response = EnvNotification::GotFunctionResponse {
            ticket,
            agent_id: agent_id.to_string(),
            json,
        };
        self.channel
            .sender
            .lock()
            .await
            .send(assistant_message_response.into())
            .await
            .map_err(|_| DispatchError::Send)
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
