#[allow(unused)]
use std::process::Command;

pub struct Pane {
    pub content: String,
    pub summary: String,
}

impl Pane {
    pub fn new() {
        todo!()
    }
}

pub fn panerate_function() {
    todo!("Create a function that keeps rate of the amount of lines being written in the current terminal and save every 10 lines, and append the lines to the file as you go to save on speed use")
}

pub fn capture_current_pane() -> Result<String, &'static String> {
    let command = Command::new("tmux")
        .args(["capture-pane", "-p", "-S", "-"])
        .output()
        .expect("Failed to capture pane");

    Ok(String::from_utf8_lossy(&command.stdout).to_string())
}
