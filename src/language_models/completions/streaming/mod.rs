use serde_json::Value;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;
use tracing_log::log::info;
pub mod error;
use crate::agents::memory::Message;
use crate::agents::Agent;
use anyhow::anyhow;
pub use error::*;
use futures::Stream;
use futures_util::StreamExt;
use serde::Deserialize;

use super::{
    anthropic::streaming::AnthropicStreamResponse, openai::streaming::OpenAiStreamResponse,
};

pub(in crate::language_models) type CompletionStream =
    Box<dyn Stream<Item = StreamResult<Value>> + Send + Unpin>;

pub(in crate::language_models) type CompletionStreamReceiver =
    tokio::sync::mpsc::Receiver<Result<CompletionStreamStatus, StreamError>>;
pub(in crate::language_models) type CompletionStreamSender =
    tokio::sync::mpsc::Sender<Result<CompletionStreamStatus, StreamError>>;

pub trait StreamResponse:
    for<'de> Deserialize<'de> + Debug + Into<CompletionStreamStatus> + Clone + Send + Sync + 'static
{
}

#[derive(Debug)]
struct CompletionStreamingThread;

#[derive(Debug)]
pub enum CompletionStreamStatus {
    Working(String),
    Finished,
}

#[derive(Debug)]
pub enum ProviderStreamHandler {
    OpenAi(StreamedCompletionHandler<OpenAiStreamResponse>),
    Anthropic(StreamedCompletionHandler<AnthropicStreamResponse>),
}

impl From<StreamedCompletionHandler<OpenAiStreamResponse>> for ProviderStreamHandler {
    fn from(value: StreamedCompletionHandler<OpenAiStreamResponse>) -> Self {
        Self::OpenAi(value)
    }
}

impl From<StreamedCompletionHandler<AnthropicStreamResponse>> for ProviderStreamHandler {
    fn from(value: StreamedCompletionHandler<AnthropicStreamResponse>) -> Self {
        Self::Anthropic(value)
    }
}

pub struct StreamedCompletionHandler<T> {
    phantom: PhantomData<T>,
    stream: Option<CompletionStream>,
    sender: Option<CompletionStreamSender>,
    receiver: CompletionStreamReceiver,
    pub message_content: String,
}

impl<T> std::fmt::Debug for StreamedCompletionHandler<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamedCompletionHandler")
            .field("stream", &"<<skipped>>")
            .field("sender", &self.sender)
            .field("phantom", &self.phantom)
            .field("receiver", &self.receiver)
            .finish()
    }
}

impl<T> From<CompletionStream> for StreamedCompletionHandler<T> {
    fn from(stream: CompletionStream) -> Self {
        let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
            tokio::sync::mpsc::channel(50);
        Self {
            phantom: PhantomData::default(),
            stream: Some(stream),
            sender: Some(tx),
            receiver: rx,
            message_content: String::new(),
        }
    }
}

impl ProviderStreamHandler {
    #[tracing::instrument("Receive tokens from completion stream", skip(self))]
    pub async fn receive(&mut self, agent: &mut Agent) -> Option<CompletionStreamStatus> {
        match self {
            Self::OpenAi(inner) => inner.receive(agent).await,
            Self::Anthropic(inner) => inner.receive(agent).await,
        }
    }
}

impl<T> StreamedCompletionHandler<T>
where
    T: StreamResponse,
{
    /// Returns tokens until finished, when finished, sends an update cache request with the full
    /// message. Best used in a while loop
    #[tracing::instrument("Receive tokens from completion stream", skip(self))]
    async fn receive(&mut self, agent: &mut Agent) -> Option<CompletionStreamStatus> {
        if self.sender.is_some() && self.stream.is_some() {
            self.spawn().ok()?;
        }
        tracing::info!("Told thread to run");
        if let Some(result) =
            tokio::time::timeout(Duration::from_millis(1000), self.receiver.recv())
                .await
                .map_err(|_| StreamError::ReceiverTimeout)
                .ok()?
        {
            match result.ok()? {
                CompletionStreamStatus::Working(token) => {
                    self.message_content.push_str(&token);
                    return Some(CompletionStreamStatus::Working(token.to_string()));
                }
                CompletionStreamStatus::Finished => {
                    tracing::info!("Stream finished with content: {}", self.message_content);
                    let message = Message::new_assistant(&self.message_content);
                    agent.cache.push(message);
                    return Some(CompletionStreamStatus::Finished);
                }
            }
        }
        None
    }

    #[tracing::instrument("Spawn completion stream thread", skip(self))]
    fn spawn(&mut self) -> Result<(), StreamError> {
        let mut stream = self.stream.take().unwrap();
        let tx = self.sender.take().unwrap();
        tracing::info!("Completion thread took stream and sender");
        let _: tokio::task::JoinHandle<Result<(), StreamError>> = tokio::spawn(async move {
            tracing::info!("Thread should run");
            loop {
                tracing::info!("Beginning of completion stream thread loop");
                match CompletionStreamingThread::poll_stream_for_type::<T>(&mut stream).await {
                    Ok(type_option) => {
                        let status: CompletionStreamStatus = match type_option {
                            Some(ref typ) => <T as Clone>::clone(&(*typ)).into(),
                            None => CompletionStreamStatus::Finished,
                        };
                        tracing::info!("Got status: {:?}", status);

                        let break_loop = match &status {
                            &CompletionStreamStatus::Finished => true,
                            _ => false,
                        };
                        tx.send(Ok(status)).await.map_err(|err| {
                            StreamError::Undefined(anyhow!("Unexpected Error: {:?}", err))
                        })?;

                        if break_loop {
                            break;
                        }
                    }
                    Err(err) => {
                        if let Err(_) = tx.send(Err(err)).await {
                            break;
                        }
                    }
                };
            }
            Ok(())
        });

        Ok(())
    }
}

impl CompletionStreamingThread {
    #[tracing::instrument(name = "Get token from stream" skip(stream))]
    async fn poll_stream_for_type<T>(stream: &mut CompletionStream) -> StreamResult<Option<T>>
    where
        T: StreamResponse,
    {
        while let Some(Ok(stream_response)) = stream.next().await {
            info!("Stream response json: {:?}", stream_response);
            let parsed_response: T = serde_json::from_value(stream_response)?;
            return Ok(Some(parsed_response));
        }

        Ok(None)
    }
}
