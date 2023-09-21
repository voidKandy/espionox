use super::FrontendRequest;
use crate::agent::Agent;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug)]
pub(super) struct AppBackend {
    runtime: tokio::runtime::Runtime,
    agent: Arc<Mutex<Agent>>,
    frontend_sender: Option<Arc<mpsc::Sender<FrontendRequest>>>,
    frontend_receiver: Option<Arc<Mutex<mpsc::Receiver<BackendCommand>>>>,
}

#[derive(Clone, Debug)]
pub(super) enum BackendCommand {
    ProcessInput(String),
}

impl From<String> for BackendCommand {
    fn from(value: String) -> Self {
        Self::ProcessInput(value.to_string())
    }
}

unsafe impl Send for BackendCommand {}
unsafe impl Sync for BackendCommand {}

impl AppBackend {
    pub fn init(
        sender: mpsc::Sender<FrontendRequest>,
        receiver: mpsc::Receiver<BackendCommand>,
    ) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");
        Self {
            runtime,
            agent: Arc::new(Mutex::new(Agent::default())),
            frontend_sender: Some(sender.into()),
            frontend_receiver: Some(Arc::new(Mutex::new(receiver))),
        }
    }

    #[tracing::instrument(name = "Process user prompt from frontend receiver in backend" skip(self))]
    pub fn process_user_prompt(&self) -> Result<(), anyhow::Error> {
        if let Some(frontend_receiver) = &self.frontend_receiver {
            let receiver = Arc::clone(&frontend_receiver);

            self.runtime.block_on(async {
                match receiver.lock().unwrap().recv().await {
                    Some(BackendCommand::ProcessInput(prompt)) => {
                        let agent = Arc::clone(&self.agent);
                        if let Some(frontend_sender) = &self.frontend_sender {
                            tracing::info!("Sender exists, buidling tokio thread...");
                            let mut stream_receiver =
                                agent.lock().unwrap().stream_prompt(&prompt).await;
                            let timeout_duration = std::time::Duration::from_millis(100);
                            while let Some(result) =
                                tokio::time::timeout(timeout_duration, stream_receiver.recv())
                                    .await
                                    .unwrap()
                            {
                                match result {
                                    Ok(response) => {
                                        tracing::info!("Response got: {}", response);
                                        frontend_sender.send(response.into()).await.unwrap();
                                    }
                                    Err(err) => {
                                        tracing::warn!("Error: {:?}", err);
                                    }
                                }
                            }
                            frontend_sender.send(FrontendRequest::Done).await.unwrap();
                        }
                    }
                    _ => {}
                }
                tracing::info!("Processed all responses, returning");
            });

            Ok(())
        } else {
            Err(anyhow::anyhow!("No reciever"))
        }
    }
}
