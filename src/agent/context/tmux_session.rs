use std::fs;
#[allow(unused)]
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Pane {
    pub content: String,
}

impl Pane {
    #[allow(unused_assignments)]
    pub fn capture(window: Option<u16>) -> Pane {
        let mut args = vec!["capture-pane", "-p", "-S"];
        let mut window_size = String::new();
        match window {
            Some(window) => {
                window_size = format!("-{}", &window.to_string());
                args.push(&window_size);
            }
            None => args.push("-"),
        }
        args.push("-E");
        args.push("-");
        let command = Command::new("tmux")
            .args(args)
            .output()
            .expect("Failed to capture pane");

        Pane {
            content: String::from_utf8_lossy(&command.stdout).to_string(),
        }
    }
    pub fn parse_pane() {}
    pub fn write_to(&self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(file, &self.content)?;
        Ok(())
    }
}
