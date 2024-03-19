use uuid::Uuid;

pub use crate::{
    agents::{
        error::AgentError,
        memory::{Message, MessageRole, MessageStack, ToMessage},
    },
    language_models::{
        completion_handler::EndpointCompletionHandler,
        openai::{completions::OpenAiResponse, functions::CustomFunction},
    },
};

use super::{EnvMessageSender, EnvRequest};

/// Handle for making requests to agents within Environment
#[derive(Debug, Clone)]
pub struct AgentHandle {
    /// Associated Agent's ID
    pub id: String,
    /// Connection to environment
    pub sender: EnvMessageSender,
}

impl From<(&str, EnvMessageSender)> for AgentHandle {
    fn from((id, sender): (&str, EnvMessageSender)) -> Self {
        Self {
            id: id.to_string(),
            sender,
        }
    }
}

impl AgentHandle {
    /// Requests an update to the handle's agent's cache
    #[tracing::instrument(name = "Send request for current state of the agent", skip(self))]
    pub async fn request_cache_push(
        &mut self,
        to_message: impl ToMessage,
        role: MessageRole,
    ) -> Result<(), AgentError> {
        let request = EnvRequest::PushToCache {
            agent_id: self.id.clone(),
            message: to_message.to_message(role),
        };
        self.sender
            .lock()
            .await
            .send(request.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        Ok(())
    }

    /// Requests the status of the given agent in the form of a cache update
    #[tracing::instrument(name = "Send request for current state of the agent", skip(self))]
    pub async fn request_state(&mut self) -> Result<Uuid, AgentError> {
        let ticket = Uuid::new_v4();
        let request = EnvRequest::GetAgentState {
            ticket,
            agent_id: self.id.to_string(),
        };
        self.sender
            .lock()
            .await
            .send(request.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        Ok(ticket)
    }
    /// Requests a cache update and a completion for agent, returns ticket number
    #[tracing::instrument(name = "Send request to prompt agent to env", skip(self))]
    pub async fn request_io_completion(&mut self, message: Message) -> Result<Uuid, AgentError> {
        let cache_change = EnvRequest::PushToCache {
            agent_id: self.id.clone(),
            message,
        };
        self.sender
            .lock()
            .await
            .send(cache_change.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        tracing::info!("Requested a cache change");

        let ticket = Uuid::new_v4();
        let completion = EnvRequest::GetCompletion {
            ticket,
            agent_id: self.id.clone(),
        };
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(completion.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        tracing::info!("Requested completion");
        Ok(ticket)
    }

    /// Requests env for streamed response, returns ticket number
    #[tracing::instrument(name = "Send request for a stream handle to env", skip(self))]
    pub async fn request_stream_completion(
        &mut self,
        message: Message,
    ) -> Result<Uuid, AgentError> {
        let cache_change = EnvRequest::PushToCache {
            agent_id: self.id.clone(),
            message,
        };
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(cache_change.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;

        let ticket = Uuid::new_v4();
        let stream_req = EnvRequest::GetCompletionStreamHandle {
            ticket,
            agent_id: self.id.clone(),
        };

        tracing::info!("Requested a stream completion from the env");
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(stream_req.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        Ok(ticket)
    }

    #[tracing::instrument(name = "Function prompt GPT API for response" skip(message, custom_function))]
    pub async fn request_function_prompt(
        &mut self,
        custom_function: CustomFunction,
        message: Message,
    ) -> Result<Uuid, AgentError> {
        let cache_change = EnvRequest::PushToCache {
            agent_id: self.id.clone(),
            message,
        };
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(cache_change.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        let function = custom_function.function();

        let ticket = Uuid::new_v4();
        let func_req = EnvRequest::GetFunctionCompletion {
            ticket,
            agent_id: self.id.clone(),
            function,
        };
        self.sender
            .try_lock()
            .expect("Failed to lock agent sender")
            .send(func_req.into())
            .await
            .map_err(|_| AgentError::EnvSend)?;
        tracing::info!("Requested a stream completion from the env");
        Ok(ticket)
    }
}
