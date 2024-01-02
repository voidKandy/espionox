use crate::environment::errors::GptError;
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
    tokio::sync::mpsc::Receiver<Result<Option<String>, StreamError>>;
pub type CompletionStreamSender = tokio::sync::mpsc::Sender<Result<Option<String>, StreamError>>;

pub struct CompletionReceiverHandler(CompletionStreamReceiver);

impl From<CompletionStreamReceiver> for CompletionReceiverHandler {
    fn from(value: CompletionStreamReceiver) -> Self {
        Self(value)
    }
}

pub struct CompletionStreamingThread(tokio::task::JoinHandle<Result<(), StreamError>>);

impl From<tokio::task::JoinHandle<Result<(), StreamError>>> for CompletionStreamingThread {
    fn from(value: tokio::task::JoinHandle<Result<(), StreamError>>) -> Self {
        Self(value)
    }
}

impl AsRef<tokio::task::JoinHandle<Result<(), StreamError>>> for CompletionStreamingThread {
    fn as_ref(&self) -> &tokio::task::JoinHandle<Result<(), StreamError>> {
        &self.0
    }
}

impl CompletionReceiverHandler {
    pub async fn receive(&mut self) -> Result<Option<String>, StreamError> {
        match self.0.recv().await {
            Some(Ok(option)) => Ok(option),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }
}

impl CompletionStreamingThread {
    pub fn spawn_poll_stream_for_tokens(
        mut stream: CompletionStream,
        tx: CompletionStreamSender,
    ) -> Self {
        let timeout = tokio::time::Duration::from_millis(100);
        let handle = tokio::spawn(async move {
            loop {
                match Self::poll_stream_for_tokens(&mut stream).await {
                    Ok(option) => match option {
                        Some(token) => {
                            tracing::info!("Token got: {}", token);
                            if let Err(_) = tx.send_timeout(Ok(Some(token)), timeout).await {
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
            Ok(())
        });

        Self::from(handle)
    }

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
