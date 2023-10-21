use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::{Command, Stdio},
};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub struct Io {
    pub i: String,
    pub o: Option<String>,
}

impl Display for Io {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("Input: {}, Output: {:?}", &self.i, &self.o,);
        write!(f, "{}", string)
    }
}
impl Io {
    pub fn run_input(&mut self) {
        let args: Vec<&str> = self.i.split_whitespace().collect();
        let out = Command::new(args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute command");
        self.o = Some(String::from_utf8_lossy(&out.stdout).to_string());
    }

    pub fn init(input: &str) -> Io {
        Io {
            i: input.to_string(),
            o: None,
        }
    }
}
