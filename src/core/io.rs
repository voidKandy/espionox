use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::{Command, Stdio},
};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Io {
    pub i: String,
    pub o: String,
}

impl Display for Io {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("Input: {}, Output: {}", &self.i, &self.o,);
        write!(f, "{}", string)
    }
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
