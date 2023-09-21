pub mod backend;
pub mod frontend;
pub mod window;

use self::{
    backend::{AppBackend, BackendCommand},
    frontend::{AppFrontend, FrontendRequest},
};
use eframe::egui;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct MainApplication {
    frontend: AppFrontend,
    backend: AppBackend,
}

impl Default for MainApplication {
    fn default() -> Self {
        let (frontend_to_backend_sender, frontend_to_backend_receiver) = mpsc::channel(100);
        let (backend_to_frontend_sender, backend_to_frontend_receiver) = mpsc::channel(100);

        let frontend = AppFrontend::init(frontend_to_backend_sender, backend_to_frontend_receiver);

        let backend = AppBackend::init(backend_to_frontend_sender, frontend_to_backend_receiver);

        Self { frontend, backend }
    }
}

impl MainApplication {
    pub fn run() -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            decorated: false,
            transparent: true,
            min_window_size: Some(egui::vec2(1280.0, 640.0)),
            initial_window_size: Some(egui::vec2(1280.0, 640.0)),
            ..Default::default()
        };
        eframe::run_native(
            "Consoxide",
            options,
            Box::new(|_cc| Box::<MainApplication>::default()),
        )
    }
}

impl eframe::App for MainApplication {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array() // Make sure we don't paint anything behind the rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        window::custom_window_frame(ctx, frame, "Consoxide", |ui| {
            ui.label("This is just the contents of the window.");
            ui.horizontal(|ui| {
                ui.vertical_centered(|ui| {
                    self.frontend.buffer_panel(ui);
                    self.frontend.current_exchange_panel(ui);
                });
                self.frontend.user_input_panel(&self.backend, ui.ctx());
                // self.agent_handler.last_message_panel(ui);
                // self.buffer_panel(ui);
                //     ui.label("egui theme:");
                //     egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });
    }
}
