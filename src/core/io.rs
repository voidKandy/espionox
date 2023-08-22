use super::Memorable;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Io {
    pub i: String,
    pub o: String,
}

impl Io {
    fn run_input(input: &str) -> String {
        let args: Vec<&str> = input.split_whitespace().collect();
        let out = Command::new(args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute command");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    pub fn new(input: &str) -> Io {
        Io {
            i: input.to_string(),
            o: Self::run_input(input),
        }
    }
}

impl Memorable for Io {
    fn memorize(&self) -> String {
        format!("Input: {}, Output: {}", &self.i, &self.o,)
    }
}
