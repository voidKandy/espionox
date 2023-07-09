use inquire::Text;
use std::collections::HashMap;
#[allow(unused)]
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Pane {
    pub name: String,
    pub contents: HashMap<String, String>,
    pub pwd: String,
    start_end_patterns: (String, String),
}

impl Pane {
    pub fn new() -> Pane {
        let default_start_pattern = String::from("===START===");
        let default_end_pattern = String::from("===END===");
        Pane {
            name: String::from("tmux-monitor:0.1"),
            contents: HashMap::new(),
            pwd: Pane::get_pwd(),
            start_end_patterns: (default_start_pattern, default_end_pattern),
        }
    }
    pub fn get_pwd() -> String {
        let pwd = Command::new("pwd")
            .output()
            .expect("Failed to get current directory");

        String::from_utf8_lossy(&pwd.stdout).trim().to_string()
    }

    fn capture_content(&self) -> String {
        let args = vec!["capture-pane", "-p", "-t", &self.name];
        let command = Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to capture pane");

        String::from_utf8_lossy(&command.stdout).to_string()
    }

    fn parse_pane(&self, content: String) -> String {
        let start_idcs: Vec<_> = content
            .match_indices(&self.start_end_patterns.0)
            .map(|(i, _)| i + self.start_end_patterns.0.len() + 1)
            .collect();
        let end_idcs: Vec<_> = content
            .match_indices(&self.start_end_patterns.1)
            .map(|(i, _)| i)
            .collect();

        content[start_idcs[start_idcs.len() - 1]..end_idcs[end_idcs.len() - 1]].to_string()
    }

    pub fn watch(&mut self) {
        loop {
            let prompt = Text::new("::: ").prompt().unwrap();
            let command = format!(
                r#"{} | awk 'BEGIN {{ print "{}"}} {{ print }} END {{ print "{}" }}'"#,
                prompt, &self.start_end_patterns.0, &self.start_end_patterns.1
            );

            let args = ["send-keys", "-t", &self.name, &command, "Enter"];
            Command::new("tmux")
                .args(args)
                .output()
                .expect("failed to execute tmux command");
            thread::sleep(Duration::from_millis(500));

            let command_output_string = self.capture_content();
            self.contents
                .insert(prompt, self.parse_pane(command_output_string));
        }
    }
}
