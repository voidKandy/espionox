use inquire::Text;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Pane {
    pub name: String,
    pub contents: HashMap<String, String>,
    pub pwd: String,
    match_patterns: (String, String),
}

impl Pane {
    pub fn new() -> Pane {
        let default_start_pattern = String::from("===START===");
        let default_end_pattern = String::from("===END===");
        Pane {
            name: String::from("tmux-monitor:0.1"),
            contents: HashMap::new(),
            pwd: Pane::get_pwd(),
            match_patterns: (default_start_pattern, default_end_pattern),
        }
    }
    pub fn get_pwd() -> String {
        let pwd = Command::new("pwd")
            .output()
            .expect("Failed to get current directory");

        String::from_utf8_lossy(&pwd.stdout).trim().to_string()
    }

    pub fn watch(&mut self) {
        let prompt = Text::new("::: ").prompt().unwrap();
        let pargs: Vec<&str> = prompt.split(" ").collect();
        let command = match pargs[0] {
            "cd" => prompt.clone(),
            _ => format!(
                r#"{} | awk 'BEGIN {{ print "{}"}} {{ print }} END {{ print "{}" }}'"#,
                prompt, &self.match_patterns.0, &self.match_patterns.1,
            ),
        };
        let args = ["send-keys", "-t", &self.name, &command, "Enter"];
        Command::new("tmux")
            .args(args)
            .output()
            .expect("failed to execute tmux command");
        thread::sleep(Duration::from_millis(500));

        let command_output_string = self.capture_content();
        self.contents
            .insert(prompt, self.get_last_output(command_output_string));
    }

    pub fn is_problematic(&self) -> bool {
        let error_substrings = ["error", "failure", "problem"];
        match self.contents.iter().last() {
            Some((_, last_out)) => error_substrings
                .iter()
                .any(|substring| last_out.to_lowercase().contains(substring)),
            None => false,
        }
    }
    fn capture_content(&self) -> String {
        let args = vec!["capture-pane", "-p", "-t", &self.name];
        let command = Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to capture pane");

        String::from_utf8_lossy(&command.stdout).to_string()
    }

    fn get_last_output(&self, content: String) -> String {
        let start_idcs: Vec<_> = content
            .match_indices(&self.match_patterns.0)
            .map(|(i, _)| i + self.match_patterns.0.len() + 1)
            .collect();
        let end_idcs: Vec<_> = content
            .match_indices(&self.match_patterns.1)
            .map(|(i, _)| i)
            .collect();

        content[start_idcs[start_idcs.len() - 1]..end_idcs[end_idcs.len() - 1]].to_string()
    }
}
