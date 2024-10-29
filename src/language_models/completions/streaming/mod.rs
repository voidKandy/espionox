use serde_json::Value;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;
use tracing::warn;
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
    pub async fn receive(
        &mut self,
        agent: &mut Agent,
    ) -> StreamResult<Option<CompletionStreamStatus>> {
        let response = match self {
            Self::OpenAi(inner) => inner.receive(agent).await,
            Self::Anthropic(inner) => inner.receive(agent).await,
        };

        tracing::warn!("got stream response:  {response:#?}");
        response
    }
}

impl<T> StreamedCompletionHandler<T>
where
    T: StreamResponse,
{
    /// Returns tokens until finished, when finished, sends an update cache request with the full
    /// message. Best used in a while loop
    #[tracing::instrument("Receive tokens from completion stream", skip(self))]
    async fn receive(&mut self, agent: &mut Agent) -> StreamResult<Option<CompletionStreamStatus>> {
        if self.sender.is_some() && self.stream.is_some() {
            tracing::info!("Telling thread to run");
            self.spawn()?;
        }
        if let Some(result) =
            tokio::time::timeout(Duration::from_millis(1000), self.receiver.recv())
                .await
                .map_err(|_| StreamError::ReceiverTimeout)?
        {
            match result? {
                CompletionStreamStatus::Working(token) => {
                    self.message_content.push_str(&token);
                    return Ok(Some(CompletionStreamStatus::Working(token.to_string())));
                }
                // CompletionStreamStatus::Error(json) => {
                //     tracing::info!("Stream recieved an error: {json:#?}");
                //     return Err(StreamError::from(json));
                // }
                CompletionStreamStatus::Finished => {
                    tracing::info!("Stream finished with content: {}", self.message_content);
                    let message = Message::new_assistant(&self.message_content);
                    agent.cache.push(message);
                    return Ok(Some(CompletionStreamStatus::Finished));
                }
            }
        }
        tracing::info!("received none");
        Ok(None)
    }

    #[tracing::instrument("Spawn completion stream thread", skip(self))]
    fn spawn(&mut self) -> Result<(), StreamError> {
        let mut stream = self.stream.take().unwrap();
        let tx = self.sender.take().unwrap();
        tokio::spawn(async move {
            loop {
                tracing::info!("Beginning of completion stream thread loop");
                match CompletionStreamingThread::poll_stream_for_type::<T>(&mut stream).await {
                    Ok(type_option) => {
                        let status: CompletionStreamStatus = match type_option {
                            Some(ret) => match ret {
                                StreamPollReturn::Ok(typ) => <T as Clone>::clone(&(typ)).into(),
                                StreamPollReturn::Err(json) => {
                                    tx.send(Err(StreamError::from(json)))
                                        .await
                                        .expect("could not send");
                                    break;
                                }
                            },
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
                        tx.send(Err(err))
                            .await
                            .expect("failed to send error message back");
                        break;
                    }
                };
            }
            tracing::info!("outside of loop");
            return Ok::<(), StreamError>(());
        });

        Ok(())
    }
}

pub enum StreamPollReturn<T> {
    Ok(T),
    Err(serde_json::Value),
}

impl<T> From<T> for StreamPollReturn<T>
where
    T: StreamResponse,
{
    fn from(value: T) -> Self {
        Self::Ok(value)
    }
}

impl<T> From<serde_json::Value> for StreamPollReturn<T> {
    fn from(value: serde_json::Value) -> Self {
        Self::Err(value)
    }
}

impl CompletionStreamingThread {
    #[tracing::instrument(name = "Get token from stream" skip(stream))]
    async fn poll_stream_for_type<T>(
        stream: &mut CompletionStream,
    ) -> StreamResult<Option<StreamPollReturn<T>>>
    where
        T: StreamResponse,
    {
        while let Some(Ok(stream_response)) = stream.next().await {
            warn!("Stream response json: {:?}", stream_response);
            match serde_json::from_value::<T>(stream_response.clone()) {
                Ok(val) => return Ok(Some(StreamPollReturn::from(val))),
                Err(err) => {
                    warn!("poll stream for type failed to coerce to T: {err:#?}");
                    return Ok(Some(StreamPollReturn::from(stream_response)));
                }
            }
        }

        Ok(None)
    }
}
