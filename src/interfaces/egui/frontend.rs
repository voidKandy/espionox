use super::{AppBackend, BackendCommand};
use crate::context::{Message, MessageVector};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug)]
pub(super) struct AppFrontend {
    pub current_exchange: CurrentExchange,
    pub active_buffer: MessageVector,
    pub backend_sender: Option<Arc<mpsc::Sender<BackendCommand>>>,
    pub backend_receiver: Option<Arc<Mutex<mpsc::Receiver<FrontendRequest>>>>,
}

#[derive(Debug, Clone)]
pub enum FrontendRequest {
    Message(String),
    Done,
}

#[derive(Default, Debug, Clone)]
pub struct CurrentExchange {
    pub user_input: String,
    pub agent_responses: Vec<FrontendRequest>,
}
unsafe impl Send for FrontendRequest {}
unsafe impl Sync for FrontendRequest {}

impl From<String> for FrontendRequest {
    fn from(str: String) -> Self {
        Self::Message(str)
    }
}

impl Into<String> for FrontendRequest {
    fn into(self) -> String {
        match self {
            Self::Message(string) => string,
            Self::Done => "Done".to_string(),
        }
    }
}

impl AppFrontend {
    const USER_COLOR: egui::Color32 = egui::Color32::from_rgb(155, 240, 255);
    const AGENT_COLOR: egui::Color32 = egui::Color32::from_rgb(128, 140, 255);
    const SYSTEM_COLOR: egui::Color32 = egui::Color32::from_rgb(228, 240, 115);

    pub fn init(
        sender: mpsc::Sender<BackendCommand>,
        receiver: mpsc::Receiver<FrontendRequest>,
    ) -> Self {
        Self {
            current_exchange: CurrentExchange::default(),
            active_buffer: MessageVector::init(),
            backend_sender: Some(sender.into()),
            backend_receiver: Some(Arc::new(Mutex::new(receiver))),
        }
    }

    pub fn current_exchange_panel(&mut self, ui: &mut egui::Ui) {
        use egui::text::LayoutJob;
        let mut job = LayoutJob::default();

        job.append(
            &format!("User: {}\n", self.current_exchange.user_input),
            0.0,
            egui::TextFormat {
                color: Self::USER_COLOR,
                ..Default::default()
            },
        );

        if let Ok(response) = self
            .backend_receiver
            .to_owned()
            .unwrap()
            .lock()
            .unwrap()
            .try_recv()
        {
            self.current_exchange.agent_responses.push(response.into());
        }

        for request in self.current_exchange.agent_responses.iter() {
            match request {
                FrontendRequest::Message(message) => job.append(
                    &format!("{}", message),
                    0.0,
                    egui::TextFormat {
                        color: Self::AGENT_COLOR,
                        ..Default::default()
                    },
                ),
                FrontendRequest::Done => job.append(
                    &format!("\n"),
                    0.0,
                    egui::TextFormat {
                        ..Default::default()
                    },
                ),
            }
        }
        ui.label(job);
    }

    // #[tracing::instrument(name = "Display user text input box and handle input" skip(ctx, self))]
    pub fn user_input_panel(&mut self, backend: &AppBackend, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("user_input").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let text_edit = egui::TextEdit::multiline(&mut self.current_exchange.user_input)
                    .desired_rows(1);
                let enter_button = egui::Button::new(">>");
                ui.add(text_edit);
                if ui.add(enter_button).clicked() {
                    if let Some(frontend_sender) = &self.backend_sender {
                        let user_input = self.current_exchange.user_input.to_owned();
                        self.active_buffer
                            .as_mut_ref()
                            .push(Message::new_standard("user", &user_input));
                        ctx.request_repaint();

                        let sender = Arc::clone(frontend_sender);
                        sender
                            .try_send(BackendCommand::from(user_input))
                            .expect("Failed to send to backend");
                        backend
                            .process_user_prompt()
                            .expect("Failed to process user prompt");
                        self.current_exchange.user_input.clear();
                    }
                }
            })
        });
    }

    pub fn buffer_panel(&self, ui: &mut egui::Ui) {
        use egui::text::LayoutJob;
        let active_buffer = self.active_buffer.as_ref().clone();
        let mut buffer_job = LayoutJob::default();

        for message in active_buffer.into_iter() {
            match message.role().as_str() {
                "user" => buffer_job.append(
                    &format!(
                        "User: {}\n",
                        message.content().expect("Failed to get ai message content")
                    ),
                    0.0,
                    egui::TextFormat {
                        color: Self::USER_COLOR,
                        ..Default::default()
                    },
                ),
                "assistant" => buffer_job.append(
                    &format!(
                        "Agent: {}\n",
                        message.content().expect("Failed to get ai message content")
                    ),
                    0.0,
                    egui::TextFormat {
                        color: Self::AGENT_COLOR,
                        ..Default::default()
                    },
                ),
                "system" => buffer_job.append(
                    &format!(
                        "System: {}\n",
                        message
                            .content()
                            .expect("Failed to get system message content")
                    ),
                    0.0,
                    egui::TextFormat {
                        color: Self::SYSTEM_COLOR,
                        ..Default::default()
                    },
                ),
                _ => {}
            }
        }

        ui.label(buffer_job);
    }
}
