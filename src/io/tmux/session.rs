use super::pane::{InSession, Pane};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process::Command;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TmuxSession {
    pub watched_pane: Pane,
    pub output_pane: Pane,
    pub io: HashMap<String, String>,
}

impl TmuxSession {
    pub fn new() -> TmuxSession {
        dotenv::dotenv().ok();
        TmuxSession {
            watched_pane: Pane::new(&env::var("WATCHED_PANE").unwrap()),
            output_pane: Pane::new(&env::var("OUTPUT_PANE").unwrap()),
            io: HashMap::new(),
            // pwd: TmuxSession::get_pwd(),
            // match_patterns: (default_start_pattern, default_end_pattern),
        }
    }
    pub fn get_pwd() -> String {
        let pwd = Command::new("pwd")
            .output()
            .expect("Failed to get current directory");

        String::from_utf8_lossy(&pwd.stdout).trim().to_string()
    }

    pub fn watch(&mut self) {
        let (p, c) = self.watched_pane.cl_io();
        self.io.insert(p, c);
    }

    pub fn is_problematic(&self) -> bool {
        let error_substrings = ["error", "failure", "problem"];
        match self.io.iter().last() {
            Some((_, last_out)) => error_substrings
                .iter()
                .any(|substring| last_out.to_lowercase().contains(substring)),
            None => false,
        }
    }

    pub fn to_out(&self, content: &str) {
        self.output_pane.print(content)
    }
}
