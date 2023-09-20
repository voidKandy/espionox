use crate::{
    agent::Agent,
    context::{Message, MessageVector},
};
use eframe::egui;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub(super) struct AgentHandler {
    agent: Agent,
    user_prompt: String,
    active_buffer: MessageVector,
    thread_sender: Option<Sender<String>>,
    thread_reciever: Option<Receiver<Result<String, anyhow::Error>>>,
    agent_responses: Option<Vec<String>>,
}

impl Default for AgentHandler {
    fn default() -> Self {
        Self {
            agent: Agent::default(),
            user_prompt: "".to_string(),
            active_buffer: MessageVector::init(),
            thread_sender: None,
            thread_reciever: None,
            agent_responses: None,
        }
    }
}

impl AgentHandler {
    fn new_user_message(&mut self, message: &str, ui: &egui::Ui) {
        self.active_buffer
            .as_mut_ref()
            .push(Message::new_standard("user", message));
        ui.ctx().request_repaint();
    }
    fn new_agent_message(&mut self, message: &str, ui: &egui::Ui) {
        self.active_buffer
            .as_mut_ref()
            .push(Message::new_standard("assistant", message));
        ui.ctx().request_repaint();
    }

    pub fn user_input_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("user_input").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                let text_edit = egui::TextEdit::singleline(&mut self.user_prompt).desired_rows(1);
                let enter_button = egui::Button::new(">>");
                ui.add(text_edit);
                if ui.add(enter_button).clicked() {
                    self.new_user_message(&self.user_prompt.to_owned(), ui);
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(async {
                            self.thread_reciever =
                                Some(self.agent.stream_prompt(&self.user_prompt).await);
                            println!("In thread");
                            self.stream_agent_response(ctx).await;
                        });
                    // let res = self.agent.stream_prompt(&self.user_prompt).await;
                    // self.new_agent_message(&res, ui);
                    self.user_prompt = "".to_string();
                }
            })
        });
    }

    pub fn buffer_panel(&mut self, ui: &mut egui::Ui) {
        use egui::text::LayoutJob;
        let mut buffer_job = LayoutJob::default();

        for message in self.active_buffer.as_ref().into_iter() {
            match message.role().as_str() {
                "user" => buffer_job.append(
                    &format!(
                        "User: {}\n",
                        message.content().expect("Failed to get ai message content")
                    ),
                    0.0,
                    egui::TextFormat {
                        color: egui::Color32::from_rgb(255, 140, 255),
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
                        color: egui::Color32::from_rgb(128, 140, 255),
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
                        color: egui::Color32::from_rgb(228, 240, 115),
                        ..Default::default()
                    },
                ),
                _ => {}
            }
        }
        ui.label(buffer_job);
    }

    pub fn last_message_panel(&mut self, ui: &mut egui::Ui) {
        use egui::text::LayoutJob;
        let mut job = LayoutJob::default();
        job.append(
            "Assistant: ",
            0.0,
            egui::TextFormat {
                color: egui::Color32::from_rgb(128, 140, 255),
                ..Default::default()
            },
        );
        if let Some(responses) = &self.agent_responses {
            responses.into_iter().for_each(|r| {
                job.append(
                    &format!("{}", r),
                    0.0,
                    egui::TextFormat {
                        color: egui::Color32::from_rgb(128, 140, 255),
                        ..Default::default()
                    },
                );
            });
        }
        ui.label(job);
    }

    async fn stream_agent_response(&mut self, ctx: &egui::Context) {
        let timeout_duration = std::time::Duration::from_millis(100);
        let mut rec = self.thread_reciever.take().expect("No Receiver");

        while let Some(result) = rec.recv().await {
            match result {
                Ok(response) => {
                    if let Some(responses) = &mut self.agent_responses {
                        responses.push(response.to_owned());
                    } else {
                        self.agent_responses = Some(vec![response.to_owned()]);
                    }
                    ctx.request_repaint();
                    println!("Responsed: {:?}", self.agent_responses.clone());
                    std::thread::sleep(timeout_duration);
                }
                Err(err) => {
                    panic!("Error: {:?}", err);
                }
            }
        }
        // let mut responses = vec![];
        // loop {
        //     match rec.recv().await {
        //         Some(result) => match result {
        //             Ok(res) => responses.push(res),
        //             Err(err) => panic!("Error: {:?}", err),
        //         },
        //         None => break,
        //     }
        // }
    }
}
