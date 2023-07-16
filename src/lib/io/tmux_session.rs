use inquire::Text;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TmuxSession {
    pub watched_pane: Pane,
    pub output_pane: Pane,
    pub io: HashMap<String, String>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pane {
    pub name: String,
    width: usize,
    match_patterns: (String, String),
    // pub pwd: String,
    // height: usize,
}
pub trait InSession {
    fn wrap_text(&self, content: &str) -> String;
    fn cl_io(&self) -> (String, String);
    fn print(&self, message: &str);
}

impl Pane {
    pub fn new(name: &str) -> Pane {
        let default_start_pattern = String::from("===START===");
        let default_end_pattern = String::from("===END===");
        Pane {
            name: name.to_string(),
            width: Self::get_width(name),
            match_patterns: (default_start_pattern, default_end_pattern),
        }
    }

    fn get_width(name: &str) -> usize {
        let args = ["display-message", "-p", "-t", name, "#{pane_width}"];

        let output = Command::new("tmux")
            .args(args)
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute command");

        String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<usize>()
            .expect("Failed to parse pane width as usize")
    }

    fn capture_content(&self) -> String {
        let args = vec!["capture-pane", "-p", "-S", "-50", "-t", &self.name];
        let command = Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to capture pane");

        String::from_utf8_lossy(&command.stdout).to_string()
    }

    fn fmt_args(&self, args: &str) -> String {
        format!(
            r#"{} | awk 'BEGIN {{ print "{}"}} {{ print }} END {{ print "{}" }}'"#,
            args, &self.match_patterns.0, &self.match_patterns.1,
        )
    }

    pub fn get_last_output(&self, content: String) -> String {
        let start_idcs: Vec<_> = content
            .match_indices(&self.match_patterns.0)
            .map(|(i, _)| i + self.match_patterns.0.len())
            .collect();
        let end_idcs: Vec<_> = content
            .match_indices(&self.match_patterns.1)
            .map(|(i, _)| i)
            .collect();
        assert!(start_idcs.iter().all(|i| i < &content.len()));
        assert!(end_idcs.iter().all(|i| i < &content.len()));
        println!(
            "Grabbed {:?} lines of output\n------------------\nStart indices\n------------------\n{:?}\n------------------\nEnd indices\n------------------\n{:?}\n------------------",
            content.len(),
            start_idcs,
            end_idcs,
        );
        content[start_idcs[start_idcs.len() - 1]..end_idcs[end_idcs.len() - 1]].to_string()
    }
}

impl InSession for Pane {
    fn wrap_text(&self, content: &str) -> String {
        content.split(' ').fold(String::new(), |acc, word| {
            let formatted_word = format!("{} ", word);
            let mut counter = 0;
            match acc.len() >= self.width {
                true => {
                    counter = acc.len() % self.width;
                }
                false => {
                    counter += acc.len();
                }
            }
            if (formatted_word.len() + counter) < self.width {
                format!("{}{}", acc, formatted_word)
            } else {
                format!("{}\n{}", acc, formatted_word)
            }
        })
    }

    fn cl_io(&self) -> (String, String) {
        // loop {
        let input = Text::new("::: ").prompt().unwrap();
        let pargs: Vec<&str> = input.split(" ").collect();

        let args = match pargs[0] {
            "cd" => input.clone(),
            _ => self.fmt_args(&input),
        };
        // println!("{:?}", &args);
        let args = ["send-keys", "-t", &self.name, &args, "Enter"];
        Command::new("tmux")
            .args(args)
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute tmux command");
        thread::sleep(Duration::from_millis(500));
        let out = self.get_last_output(self.capture_content());
        (input, out)
    }

    fn print(&self, message: &str) {
        let command = format!("echo '{}'", self.wrap_text(message));
        let args = vec!["run-shell", "-b", "-d", "0", "-t", &self.name, &command];
        Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to print to pane");
    }
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
