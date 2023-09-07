use crate::agent::Agent;
use eframe::egui;

pub(super) struct AgentApp {
    agent: Agent,
    user_prompt: String,
}
impl Default for AgentApp {
    fn default() -> Self {
        Self {
            agent: Agent::default(),
            user_prompt: "".to_string(),
        }
    }
}
impl eframe::App for AgentApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("agent_info").show(ctx, |ui| {
            ui.heading(self.agent.info_display_string());
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let ai_responses: Vec<String> = self
                .agent
                .context
                .buffer
                .as_ref()
                .iter()
                .filter(|message| message.role() == "assistant")
                .map(|message| message.content().to_string())
                .collect();
            ui.label(ai_responses.join("\n"));
        });

        egui::TopBottomPanel::bottom("user_input").show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.text_edit_multiline(&mut self.user_prompt);
                if ui.button(">>").clicked() {
                    _ = self.agent.prompt(&self.user_prompt);
                }
            });
        });
    }
}

impl AgentApp {
    pub fn message_window(&mut self, ctx: &egui::Context) {
        use egui::text::LayoutJob;
        let mut job = LayoutJob::default();
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut ai_responses: Vec<String> = Vec::new();
            let mut user_inputs: Vec<String> = Vec::new();

            for message in self.agent.context.buffer.as_ref().into_iter() {
                if message.role() == "assistant" {
                    ai_responses.push(message.content().to_string());
                } else if message.role() == "user" {
                    user_inputs.push(message.content().to_string());
                }
            }
            job.append(
                &format!("User: {}", user_inputs.join("\n")),
                0.0,
                egui::TextFormat {
                    color: egui::Color32::from_rgb(255, 140, 255),
                    ..Default::default()
                },
            );
            job.append(
                &format!("Assistant: {}", ai_responses.join("\n")),
                0.0,
                egui::TextFormat {
                    color: egui::Color32::from_rgb(128, 140, 255),
                    ..Default::default()
                },
            );
            ui.label(job);
        });
    }
}
