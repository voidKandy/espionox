use crate::environment::{
    agent::{
        language_models::openai::{
            functions::Function, gpt::streaming_utils::StreamedCompletionHandler,
        },
        memory::MessageVector,
    },
    Message,
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
    Finish,
}

#[derive(Debug)]
pub enum EnvNotification {
    CacheUpdate {
        agent_id: String,
        cache: MessageVector,
    },
    GotCompletionResponse {
        ticket: Uuid,
        agent_id: String,
        message: Message,
    },
    GotFunctionResponse {
        ticket: Uuid,
        agent_id: String,
        json: Value,
    },
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

impl EnvNotification {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            Self::CacheUpdate { agent_id, .. } => Some(agent_id),
            Self::GotStreamHandle { agent_id, .. } => Some(agent_id),
            Self::GotCompletionResponse { agent_id, .. } => Some(agent_id),
            Self::GotFunctionResponse { agent_id, .. } => Some(agent_id),
        }
    }
    pub fn ticket_number(&self) -> Option<Uuid> {
        match self {
            Self::CacheUpdate { .. } => None,
            Self::GotStreamHandle { ticket, .. } => Some(*ticket),
            Self::GotCompletionResponse { ticket, .. } => Some(*ticket),
            Self::GotFunctionResponse { ticket, .. } => Some(*ticket),
        }
    }

    /// Consumes self & returns notification body
    pub fn extract_body(self) -> NotificationBody {
        match self {
            EnvNotification::CacheUpdate { cache, .. } => NotificationBody::MessageVector(cache),
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

pub enum NotificationBody {
    StreamedCompletionHandler(ThreadSafeStreamCompletionHandler),
    JsonValue(Value),
    SingleMessage(Message),
    MessageVector(MessageVector),
}

impl TryInto<Message> for NotificationBody {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Message, Self::Error> {
        match self {
            NotificationBody::SingleMessage(m) => Ok(m),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}

impl TryInto<MessageVector> for NotificationBody {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<MessageVector, Self::Error> {
        match self {
            NotificationBody::MessageVector(m) => Ok(m),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}

impl TryInto<Value> for NotificationBody {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Value, Self::Error> {
        match self {
            NotificationBody::JsonValue(v) => Ok(v),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}

impl TryInto<ThreadSafeStreamCompletionHandler> for NotificationBody {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<ThreadSafeStreamCompletionHandler, Self::Error> {
        match self {
            NotificationBody::StreamedCompletionHandler(h) => Ok(h),
            _ => Err(anyhow!("Wrong notification type")),
        }
    }
}
