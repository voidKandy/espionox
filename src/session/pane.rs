use std::fs;
use std::io;
#[allow(unused)]
use std::process::Command;

pub struct Pane {
    pub content: String,
}

impl Pane {
    pub fn capture() -> Pane {
        let command = Command::new("tmux")
            .args(["capture-pane", "-p", "-S", "-"])
            .output()
            .expect("Failed to capture pane");

        Pane {
            content: String::from_utf8_lossy(&command.stdout).to_string(),
        }
    }
    pub fn write_to(&self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(file, &self.content)?;
        Ok(())
    }
}
