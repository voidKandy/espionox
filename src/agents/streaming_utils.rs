use super::AgentError;
use crate::language_models::openai::{
    errors::GptError,
    gpt::{CompletionStream, StreamResponse},
};

use futures_util::StreamExt;

pub type CompletionStreamReceiver = tokio::sync::mpsc::Receiver<Result<Option<String>, AgentError>>;
pub type CompletionStreamSender = tokio::sync::mpsc::Sender<Result<Option<String>, AgentError>>;

pub struct CompletionReceiverHandler(CompletionStreamReceiver);

impl From<CompletionStreamReceiver> for CompletionReceiverHandler {
    fn from(value: CompletionStreamReceiver) -> Self {
        Self(value)
    }
}

pub struct CompletionStreamingThread(tokio::task::JoinHandle<Result<(), AgentError>>);

impl From<tokio::task::JoinHandle<Result<(), AgentError>>> for CompletionStreamingThread {
    fn from(value: tokio::task::JoinHandle<Result<(), AgentError>>) -> Self {
        Self(value)
    }
}

impl AsRef<tokio::task::JoinHandle<Result<(), AgentError>>> for CompletionStreamingThread {
    fn as_ref(&self) -> &tokio::task::JoinHandle<Result<(), AgentError>> {
        &self.0
    }
}

impl CompletionReceiverHandler {
    pub async fn receive(&mut self) -> Result<Option<String>, AgentError> {
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
                            GptError::Recoverable(_) => AgentError::RetryError,
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
        while let Some(Ok(chunk)) = stream.next().await {
            match StreamResponse::from_byte_chunk(chunk).await {
                Ok(response_option) => {
                    if let Some(stream_response) = response_option {
                        let parsed_response = stream_response.parse().unwrap();
                        return Ok(Some(parsed_response));
                    } else {
                        return Ok(None);
                    }
                }
                Err(err) => return Err(err),
            };
        }

        Ok(None)
    }
}
