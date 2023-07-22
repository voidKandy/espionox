use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Commander {
    pub history: Vec<Io>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Io(pub String, pub String);

impl Io {
    pub fn new(input: &str) -> Io {
        let out = Commander::run_input(&input);
        Io(input.to_string(), out)
    }
}

impl Commander {
    pub fn new() -> Commander {
        Commander {
            history: Vec::new(),
        }
    }

    fn run_input(input: &str) -> String {
        let args: Vec<&str> = input.split(" ").collect();
        let out = Command::new(args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute tmux command");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    pub fn update(&mut self, io: Io) {
        self.history.push(io);
    }
}
