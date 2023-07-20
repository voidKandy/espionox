use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Commander {
    pub history: Vec<Io>,
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Io(pub String, pub String);

impl Io {
    fn new() -> Self {
        Io(String::new(), String::new())
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

    fn get_io(input: &str) -> Io {
        let out = Self::run_input(&input);
        thread::sleep(Duration::from_millis(500));
        Io(input.to_string(), out)
    }

    fn update(&mut self, io: Io) {
        self.history.push(io);
    }
}
