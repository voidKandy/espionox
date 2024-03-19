use crate::{
    agents::memory::{Message, MessageStack},
    language_models::openai::{
        completions::streaming::StreamedCompletionHandler, functions::Function,
    },
};
use anyhow::anyhow;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use uuid::Uuid;

pub type EnvMessageSender = Arc<Mutex<Sender<EnvMessage>>>;
pub type EnvMessageReceiver = Arc<Mutex<Receiver<EnvMessage>>>;
pub type ThreadSafeStreamCompletionHandler = Arc<Mutex<StreamedCompletionHandler>>;

#[derive(Debug, Clone)]
pub struct EnvChannel {
    pub(crate) sender: EnvMessageSender,
    pub(crate) receiver: EnvMessageReceiver,
}

#[derive(Debug)]
pub enum EnvMessage {
    Request(EnvRequest),
    Response(EnvNotification),
    Finish,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnvRequest {
    GetCompletion {
        ticket: Uuid,
        agent_id: String,
    },
    GetCompletionStreamHandle {
        ticket: Uuid,
        agent_id: String,
    },
    GetFunctionCompletion {
        ticket: Uuid,
        agent_id: String,
        function: Function,
    },
    PushToCache {
        agent_id: String,
        message: Message,
    },
    ResetCache {
        agent_id: String,
        keep_sys_message: bool,
    },
    GetAgentState {
        ticket: Uuid,
        agent_id: String,
    },
    Finish,
}

#[derive(Debug)]
pub enum EnvNotification {
    AgentStateUpdate {
        ticket: Uuid,
        agent_id: String,
        cache: MessageStack,
    },

    /// Returned by a request for an io completion
    GotCompletionResponse {
        ticket: Uuid,
        agent_id: String,
        message: Message,
    },

    /// Returned by a request for a function completion
    GotFunctionResponse {
        ticket: Uuid,
        agent_id: String,
        json: Value,
    },

    /// Returned by a request for a stream completion
    GotStreamHandle {
        ticket: Uuid,
        agent_id: String,
        handler: ThreadSafeStreamCompletionHandler,
    },
}

impl Into<EnvMessage> for EnvRequest {
    fn into(self) -> EnvMessage {
        EnvMessage::Request(self)
    }
}

impl Into<EnvMessage> for EnvNotification {
    fn into(self) -> EnvMessage {
        EnvMessage::Response(self)
    }
}

impl TryInto<EnvNotification> for EnvMessage {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<EnvNotification, Self::Error> {
        match self {
            EnvMessage::Response(noti) => Ok(noti),
            _ => Err(anyhow!("EnvMessage is wrong type")),
        }
    }
}

impl TryInto<EnvRequest> for EnvMessage {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<EnvRequest, Self::Error> {
        match self {
            EnvMessage::Request(req) => Ok(req),
            _ => Err(anyhow!("EnvMessage is wrong type")),
        }
    }
}

impl
    From<(
        Arc<Mutex<Sender<EnvMessage>>>,
        Arc<Mutex<Receiver<EnvMessage>>>,
    )> for EnvChannel
{
    fn from(
        (sender, receiver): (
            Arc<Mutex<Sender<EnvMessage>>>,
            Arc<Mutex<Receiver<EnvMessage>>>,
        ),
    ) -> Self {
        Self { sender, receiver }
    }
}

impl EnvMessage {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            Self::Request(req) => req.agent_id(),
            Self::Response(noti) => noti.agent_id(),
            Self::Finish => None,
        }
    }
    pub fn ticket_number(&self) -> Option<Uuid> {
        match self {
            Self::Request(req) => req.ticket_number(),
            Self::Response(noti) => noti.ticket_number(),
            Self::Finish => None,
        }
    }
}

impl EnvRequest {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            EnvRequest::Finish => None,
            EnvRequest::ResetCache { agent_id, .. } => Some(agent_id),
            EnvRequest::PushToCache { agent_id, .. } => Some(agent_id),
            EnvRequest::GetCompletion { agent_id, .. } => Some(agent_id),
            EnvRequest::GetFunctionCompletion { agent_id, .. } => Some(agent_id),
            EnvRequest::GetCompletionStreamHandle { agent_id, .. } => Some(agent_id),
            EnvRequest::GetAgentState { agent_id, .. } => Some(agent_id),
        }
    }
    pub fn ticket_number(&self) -> Option<Uuid> {
        match self {
            EnvRequest::Finish => None,
            EnvRequest::ResetCache { .. } => None,
            EnvRequest::PushToCache { .. } => None,
            EnvRequest::GetCompletion { ticket, .. } => Some(*ticket),
            EnvRequest::GetFunctionCompletion { ticket, .. } => Some(*ticket),
            EnvRequest::GetCompletionStreamHandle { ticket, .. } => Some(*ticket),
            EnvRequest::GetAgentState { ticket, .. } => Some(*ticket),
        }
    }
}

impl EnvNotification {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            EnvNotification::AgentStateUpdate { agent_id, .. } => Some(agent_id),
            EnvNotification::GotStreamHandle { agent_id, .. } => Some(agent_id),
            EnvNotification::GotCompletionResponse { agent_id, .. } => Some(agent_id),
            EnvNotification::GotFunctionResponse { agent_id, .. } => Some(agent_id),
        }
    }
    pub fn ticket_number(&self) -> Option<Uuid> {
        match self {
            EnvNotification::AgentStateUpdate { ticket, .. } => Some(*ticket),
            EnvNotification::GotStreamHandle { ticket, .. } => Some(*ticket),
            EnvNotification::GotCompletionResponse { ticket, .. } => Some(*ticket),
            EnvNotification::GotFunctionResponse { ticket, .. } => Some(*ticket),
        }
    }

    /// Consumes self & returns notification body
    pub fn extract_body(&self) -> NotificationBody {
        match self {
            EnvNotification::AgentStateUpdate { cache, .. } => {
                NotificationBody::MessageStack(cache)
            }
            EnvNotification::GotCompletionResponse { message, .. } => {
                NotificationBody::SingleMessage(message)
            }
            EnvNotification::GotFunctionResponse { json, .. } => NotificationBody::JsonValue(json),
            EnvNotification::GotStreamHandle { handler, .. } => {
                NotificationBody::StreamedCompletionHandler(handler)
            }
        }
    }
}

pub enum NotificationBody<'b> {
    StreamedCompletionHandler(&'b ThreadSafeStreamCompletionHandler),
    JsonValue(&'b Value),
    SingleMessage(&'b Message),
    MessageStack(&'b MessageStack),
}

impl<'b> TryInto<&'b Message> for NotificationBody<'b> {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<&'b Message, Self::Error> {
        match self {
            NotificationBody::SingleMessage(m) => Ok(m),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}

impl<'b> TryInto<&'b MessageStack> for NotificationBody<'b> {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<&'b MessageStack, Self::Error> {
        match self {
            NotificationBody::MessageStack(m) => Ok(m),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}

impl<'b> TryInto<&'b Value> for NotificationBody<'b> {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<&'b Value, Self::Error> {
        match self {
            NotificationBody::JsonValue(v) => Ok(v),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}

impl<'b> TryInto<&'b ThreadSafeStreamCompletionHandler> for NotificationBody<'b> {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<&'b ThreadSafeStreamCompletionHandler, Self::Error> {
        match self {
            NotificationBody::StreamedCompletionHandler(h) => Ok(h),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}
