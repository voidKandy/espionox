pub mod agent;
pub mod errors;

use crate::Agent;
use reqwest::Client;
use std::{collections::VecDeque, sync::Arc};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex, MutexGuard, RwLock, RwLockWriteGuard,
    },
    task::JoinHandle,
};
use uuid::Uuid;

use self::{
    agent::{
        language_models::{openai::gpt::GptResponse, GptError, LanguageModel},
        memory::{Message, MessageVector},
        AgentHandle,
    },
    errors::{AgentError, EnvError},
};
use std::collections::HashMap;

pub type EnvMessageSender = Arc<Mutex<Sender<EnvMessage>>>;
pub type EnvMessageReceiver = Arc<Mutex<Receiver<EnvMessage>>>;

#[derive(Debug, Clone)]
pub struct EnvChannel {
    sender: EnvMessageSender,
    receiver: EnvMessageReceiver,
}

pub type AgentHashMap = HashMap<String, Agent>;

#[derive(Debug)]
pub struct Environment {
    pub id: String,
    pub dispatch: Arc<RwLock<Dispatch>>,
    pub handle: Option<EnvThreadHandle>,
}

#[derive(Debug)]
pub struct EnvThreadHandle(JoinHandle<Result<(), EnvError>>);

#[derive(Debug)]
pub struct Dispatch {
    api_key: Option<String>,
    channel: EnvChannel,
    agents: AgentHashMap,
    response_stack: VecDeque<EnvResponse>,
}

#[derive(Debug)]
pub enum EnvMessage {
    Request(EnvRequest),
    Response(EnvResponse),
}

#[derive(Debug)]
pub enum EnvRequest {
    PromptAgent { agent_id: String, message: Message },
}

#[derive(Debug, Clone)]
pub enum EnvResponse {
    // GotCompletion { agent_id: String, response: String },
    ChangedCache { agent_id: String, message: Message },
}

impl Into<EnvMessage> for EnvRequest {
    fn into(self) -> EnvMessage {
        EnvMessage::Request(self)
    }
}

impl Into<EnvMessage> for EnvResponse {
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

impl EnvThreadHandle {
    #[tracing::instrument(name = "Spawn EnvThreadHandle")]
    async fn spawn(dispatch: Arc<RwLock<Dispatch>>) -> Self {
        let handle: JoinHandle<Result<(), EnvError>> = tokio::spawn(async move {
            let dispatch = dispatch.write().await;
            tracing::info!("Dispatch state: {:?}", dispatch);
            EnvThreadHandle::main_loop(dispatch).await?;
            Ok(())
        });
        Self(handle)
    }

    #[tracing::instrument(name = "Dispatch main loop")]
    pub async fn main_loop(mut dispatch: RwLockWriteGuard<'_, Dispatch>) -> Result<(), EnvError> {
        let receiver = Arc::clone(&dispatch.channel.receiver);
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
                    EnvMessage::Response(res) => {
                        tracing::info!("Dispatch received response: {:?}", res);
                        dispatch.handle_response(res);
                    }
                }
            }
        }
    }
}

impl Dispatch {
    fn get_agent_by_id(&mut self, id: &str) -> Option<&mut Agent> {
        if let Some(agent) = self.agents.get_mut(id) {
            return Some(agent);
        }
        None
    }

    async fn push_to_agent_cache(
        agent: &mut Agent,
        message: Message,
        sender: &EnvMessageSender,
    ) -> Result<(), EnvError> {
        let agent_id = &agent.id;
        agent.cache.push(message.clone());
        sender
            .try_lock()
            .expect("Failed to lock sender")
            .send(
                EnvResponse::ChangedCache {
                    agent_id: agent_id.clone(),
                    message,
                }
                .into(),
            )
            .await
            .map_err(|_| EnvError::Send)?;
        Ok(())
    }

    async fn handle_request(&mut self, req: EnvRequest) -> Result<(), EnvError> {
        let sender_clone = Arc::clone(&self.channel.sender);
        let api_key = &self.api_key.clone();
        match req {
            EnvRequest::PromptAgent { agent_id, message } => {
                if let Some(agent) = self.get_agent_by_id(&agent_id) {
                    Self::push_to_agent_cache(agent, message, &sender_clone)
                        .await
                        .expect("Failed to push to agent cache");
                    let completion_fn = agent.model.io_completion_fn();
                    let payload = &(&agent.cache).into();
                    if let Some(key) = api_key {
                        let client = Client::new();
                        let response = completion_fn(&client, &key, payload, &agent.model).await?;
                        let res_str = agent.handle_completion_response(response)?;
                        let message =
                            Message::new(agent::memory::messages::MessageRole::User, &res_str);
                        Self::push_to_agent_cache(agent, message, &sender_clone)
                            .await
                            .expect("Failed to push to agent cache");
                        Ok(())
                    } else {
                        Err(EnvError::Request("No api key".to_string()))
                    }
                } else {
                    Err(EnvError::Request("No agent by given id".to_string()))
                }
            }
        }
    }

    fn handle_response(&mut self, res: EnvResponse) {
        self.response_stack.push_front(res)
    }
}

impl Environment {
    /// Using the ID of an agent, get a it's handle
    pub async fn get_agent_handle(&self, id: &str) -> Option<AgentHandle> {
        let dispatch = self.dispatch.read().await;
        if let Some(agent) = dispatch.agents.get(id) {
            let sender = Arc::clone(&dispatch.channel.sender);
            let id = agent.id.as_str();
            let handle = AgentHandle::from((id, sender));
            return Some(handle);
        }
        None
    }

    /// Create new agent & insert
    pub async fn insert_agent(&mut self, agent: Agent) {
        let mut dispatch = self.dispatch.write().await;
        let id = agent.id.clone();
        dispatch.agents.insert(id, agent);
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
        let channel = EnvChannel::from((sender, receiver));

        let dispatch = Dispatch {
            channel,
            api_key: api_key.map(|k| k.to_string()),
            agents: HashMap::new(),
            response_stack: VecDeque::new(),
        };
        let dispatch = Arc::new(RwLock::new(dispatch));

        Self {
            id,
            dispatch,
            handle: None,
        }
    }

    /// Spawns env thread handle
    pub async fn run(&mut self) {
        let dispatch_clone = Arc::clone(&self.dispatch);
        let handle = EnvThreadHandle::spawn(dispatch_clone).await;
        self.handle = Some(handle);
    }

    pub async fn get_event_stack(&self) -> VecDeque<EnvResponse> {
        self.dispatch.read().await.response_stack.clone()
    }
}
