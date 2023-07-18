use inquire::Text;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Pane {
    pub name: String,
    pub content: String,
    width: usize,
    match_patterns: (String, String),
    // pub pwd: String,
    // height: usize,
}
pub trait InSession {
    fn run_input(&self, input: String);
    fn cl_io(&self) -> (String, String);
    fn last_io(&self) -> String;
    fn wrap_text(&self, content: &str) -> String;
    fn print(&self, message: &str);
}

impl Pane {
    pub fn new(name: &str) -> Pane {
        let default_start_pattern = String::from("===START===");
        let default_end_pattern = String::from("===END===");
        Pane {
            name: name.to_string(),
            content: String::new(),
            width: Self::get_width(name),
            match_patterns: (default_start_pattern, default_end_pattern),
        }
        .borrow_mut()
        .capture_content()
    }

    fn capture_content(&mut self) -> Self {
        let args = vec![
            "capture-pane",
            "-p",
            "-S",
            "-50",
            "-E",
            "-0",
            "-t",
            &self.name,
        ];
        let command = Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to capture pane");

        Pane {
            name: self.name.to_owned(),
            content: String::from_utf8_lossy(&command.stdout).to_string(),
            width: self.width,
            match_patterns: self.match_patterns.to_owned(),
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

    fn fmt_input(&self, args: &str) -> String {
        format!(
            r#"{} | awk 'BEGIN {{ print "{}"}} {{ print }} END {{ print "{}" }}'"#,
            args, &self.match_patterns.0, &self.match_patterns.1,
        )
    }
}

impl InSession for Pane {
    fn run_input(&self, input: String) {
        let pargs: Vec<&str> = input.split(" ").collect();
        let args = match pargs[0] {
            "cd" => input.to_owned(),
            _ => self.fmt_input(&input.to_owned()),
        };
        println!("{:?}", &args);
        let args = ["send-keys", "-t", &self.name, &args, "Enter"];
        Command::new("tmux")
            .args(args)
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute tmux command");
    }

    fn cl_io(&self) -> (String, String) {
        // loop {
        let input = Text::new("::: ").prompt().unwrap();
        self.run_input(input.to_string());
        thread::sleep(Duration::from_millis(500));
        let out = self.last_io();
        (input, out)
    }

    fn last_io(&self) -> String {
        let start_idcs: Vec<_> = self
            .content
            .match_indices(&self.match_patterns.0)
            .map(|(i, _)| i + self.match_patterns.0.len())
            .collect();
        let end_idcs: Vec<_> = self
            .content
            .match_indices(&self.match_patterns.1)
            .map(|(i, _)| i)
            .collect();
        assert!(start_idcs.iter().all(|i| i < &self.content.len()));
        assert!(end_idcs.iter().all(|i| i < &self.content.len()));
        println!(
            "Grabbed {:?} lines of output\n------------------\nStart indices\n------------------\n{:?}\n------------------\nEnd indices\n------------------\n{:?}\n------------------",
            &self.content.len(),
            start_idcs,
            end_idcs,
        );
        self.content[start_idcs[start_idcs.len() - 1]..end_idcs[end_idcs.len() - 1]].to_string()
    }

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

    fn print(&self, message: &str) {
        let command = format!("echo '{}'", self.wrap_text(message));
        let args = vec!["run-shell", "-b", "-d", "0", "-t", &self.name, &command];
        Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to print to pane");
    }
}
