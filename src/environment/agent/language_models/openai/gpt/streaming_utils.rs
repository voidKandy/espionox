use std::sync::{Arc, Mutex};

use crate::environment::dispatch::{EnvMessageSender, EnvRequest};
use crate::environment::{
    agent::memory::{messages::MessageRole, Message},
    errors::GptError,
};
use crate::errors::error_chain_fmt;
use futures::Stream;
use futures_util::StreamExt;
use reqwest_streams::error::StreamBodyError;
use serde::{Deserialize, Serialize};

pub type CompletionStream =
    Box<dyn Stream<Item = Result<StreamResponse, StreamBodyError>> + Send + Unpin>;

#[derive(Debug, Deserialize, Clone)]
pub struct StreamResponse {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamChoice {
    pub delta: StreamDelta,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(thiserror::Error)]
pub enum StreamError {
    #[error(transparent)]
    Undefined(#[from] anyhow::Error),
    GptError(#[from] GptError),
    RetryError,
}

impl std::fmt::Debug for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type CompletionStreamReceiver =
    tokio::sync::mpsc::Receiver<Result<CompletionStreamStatus, StreamError>>;
pub type CompletionStreamSender =
    tokio::sync::mpsc::Sender<Result<CompletionStreamStatus, StreamError>>;

#[derive(Debug)]
pub struct StreamedCompletionHandler {
    run_thread: Arc<Mutex<bool>>,
    receiver: CompletionStreamReceiver,
}

pub enum CompletionStreamStatus {
    Working(String),
    Finished,
}

impl From<CompletionStreamReceiver> for StreamedCompletionHandler {
    fn from(receiver: CompletionStreamReceiver) -> Self {
        let run_thread = Arc::new(Mutex::new(false));
        Self {
            run_thread,
            receiver,
        }
    }
}

#[derive(Debug)]
pub struct CompletionStreamingThread;

impl StreamedCompletionHandler {
    #[tracing::instrument("Spawn completion stream thread", skip(self, stream, tx))]
    pub fn spawn(
        &mut self,
        mut stream: CompletionStream,
        tx: CompletionStreamSender,
    ) -> Result<(), StreamError> {
        let timeout = tokio::time::Duration::from_millis(100);
        let should_run = Arc::clone(&self.run_thread);
        let _: tokio::task::JoinHandle<Result<(), StreamError>> = tokio::spawn(async move {
            if *should_run.lock().unwrap() {
                loop {
                    tracing::info!("Completion stream thread running");
                    match CompletionStreamingThread::poll_stream_for_tokens(&mut stream).await {
                        Ok(status) => match status {
                            Some(ref token) => {
                                tracing::info!("Token got: {}", token);
                                if let Err(_) = tx
                                    .send_timeout(
                                        Ok(CompletionStreamStatus::Working(token.to_string())),
                                        timeout,
                                    )
                                    .await
                                {
                                    break;
                                }
                            }
                            None => {
                                break;
                            }
                        },
                        Err(err) => {
                            let error = match err {
                                GptError::Recoverable => StreamError::RetryError,
                                _ => err.into(),
                            };

                            if let Err(_) = tx.send_timeout(Err(error), timeout).await {
                                break;
                            }
                        }
                    };
                }
            } else {
            }
            Ok(())
        });

        Ok(())
    }

    /// Returns tokens until finished, when finished, sends an update cache request with the full
    /// message. Best used in a while loop
    #[tracing::instrument("Receive tokens from completion stream", skip(self, sender))]
    pub async fn receive(
        &mut self,
        agent_id: &str,
        sender: EnvMessageSender,
    ) -> Result<CompletionStreamStatus, StreamError> {
        self.run_thread = Arc::new(Mutex::new(true));
        let mut message_content = String::new();
        if let Some(result) = self.receiver.recv().await {
            match result? {
                CompletionStreamStatus::Working(token) => {
                    message_content.push_str(&token);
                    return Ok(CompletionStreamStatus::Working(token.to_string()));
                }
                CompletionStreamStatus::Finished => {
                    let message = Message::new(MessageRole::Assistant, &message_content);
                    sender
                        .lock()
                        .await
                        .send(
                            EnvRequest::UpdateCache {
                                agent_id: agent_id.to_string(),
                                message,
                            }
                            .into(),
                        )
                        .await
                        .map_err(|err| StreamError::Undefined(err.into()))?
                }
            }
        }
        Ok(CompletionStreamStatus::Finished)
    }
}

impl CompletionStreamingThread {
    #[tracing::instrument(name = "Get token from stream" skip(stream))]
    async fn poll_stream_for_tokens(
        stream: &mut CompletionStream,
    ) -> Result<Option<String>, GptError> {
        while let Some(Ok(stream_response)) = stream.next().await {
            let parsed_response = stream_response.parse().unwrap();
            return Ok(Some(parsed_response));
        }

        Ok(None)
    }
}
