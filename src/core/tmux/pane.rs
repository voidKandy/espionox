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
}

pub trait InSession {
    fn run_input(&self, input: String) -> String;
    fn cl_io(&self) -> (String, String);
    fn wrap_text(&self, content: &str) -> String;
    fn print(&self, message: &str);
}

impl Pane {
    pub fn new(name: &str) -> Pane {
        Pane {
            name: name.to_string(),
            content: String::new(),
            width: Self::get_width(name),
        }
        .borrow_mut()
        .capture_content()
    }

    fn capture_content(&mut self) -> Self {
        let args = vec![
            "capture-pane",
            "-p",
            "-S",
            "0",
            "-E",
            "10",
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
}

impl InSession for Pane {
    fn run_input(&self, input: String) -> String {
        let args: Vec<&str> = input.split(" ").collect();
        let out = Command::new(args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute tmux command");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn cl_io(&self) -> (String, String) {
        // loop {
        let input = Text::new("::: ").prompt().unwrap();
        let out = self.run_input(input.to_string());
        thread::sleep(Duration::from_millis(500));
        // let out = self.last_io();
        (input, out)
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
