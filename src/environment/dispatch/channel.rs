use crate::environment::{
    agent::language_models::openai::gpt::streaming_utils::StreamedCompletionHandler, Message,
};
use std::sync::{Arc, RwLock};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};
use uuid::Uuid;

pub type EnvMessageSender = Arc<Mutex<Sender<EnvMessage>>>;
pub type EnvMessageReceiver = Arc<Mutex<Receiver<EnvMessage>>>;

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
    PromptAgent {
        ticket: Uuid,
        agent_id: String,
        message: Message,
    },
    StreamPromptAgent {
        ticket: Uuid,
        agent_id: String,
        message: Message,
    },
    UpdateCache {
        agent_id: String,
        message: Message,
    },
    Finish,
}

#[derive(Debug)]
pub enum EnvNotification {
    ChangedCache {
        agent_id: String,
        message: Message,
    },
    GotAssistantMessageResponse {
        ticket: Uuid,
        agent_id: String,
        message: Message,
    },
    GotStreamHandle {
        ticket: Uuid,
        agent_id: String,
        handler: StreamedCompletionHandler,
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
            Self::ChangedCache { agent_id, .. } => Some(agent_id),
            Self::GotStreamHandle { agent_id, .. } => Some(agent_id),
            Self::GotAssistantMessageResponse { agent_id, .. } => Some(agent_id),
        }
    }
    pub fn ticket_number(&self) -> Option<Uuid> {
        match self {
            Self::ChangedCache { .. } => None,
            Self::GotStreamHandle { ticket, .. } => Some(*ticket),
            Self::GotAssistantMessageResponse { ticket, .. } => Some(*ticket),
        }
    }
}
