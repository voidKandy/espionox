use super::walk::{Directory, File};
use crate::agent::handler::{AgentHandler, SpecialAgent};
use serde_json::{json, Value};

pub struct Context {
    pub messages: Vec<Value>,
    pub files: Vec<File>,
    pub directories: Vec<Directory>,
    pub guidance: Option<String>,
}

pub trait Contextual {
    fn init_context(&self, prompt: Option<&str>) -> Context;
    fn append_message(&self, context: &mut Vec<Value>, role: &str, content: &str);
    fn append_files_to_messages(&self, context: &mut Vec<Value>, files: Vec<File>, message: &str);
}
pub trait Operations {
    fn parse_response(&self, response: Value) -> Option<Vec<String>>;
}

impl Contextual for SpecialAgent {
    fn init_context(&self, prompt: Option<&str>) -> Context {
        let mut messages: Vec<Value> = vec![];
        if let Some(prompt) = prompt {
            messages.push(json!({"role": "system", "content": prompt}));
        };
        Context {
            messages: vec![],
            files: vec![],
            directories: vec![],
            guidance: None,
        }
    }
    fn append_message(&self, context: &mut Vec<Value>, role: &str, content: &str) {
        context.push(json!({"role": role, "content": content}))
    }
    fn append_files_to_messages(&self, context: &mut Vec<Value>, files: Vec<File>, message: &str) {
        let mut payload = vec![];
        files.iter().for_each(|f| {
            payload.push(format!(
                "FilePath: {}, Content: {}",
                &f.filepath.display(),
                &f.summary
            ));
        });
        self.append_message(
            context,
            "system",
            &format!("{} Files: {}", message, payload.join(",")),
        )
    }
}

impl Operations for AgentHandler {
    fn parse_response(&self, response: Value) -> Option<Vec<String>> {
        let arguments = response.get("arguments")?.as_str()?;
        let parsed_arguments = serde_json::from_str::<Value>(arguments).ok()?;
        let commands = parsed_arguments.get("commands")?.as_array()?;
        let command_strings = commands
            .iter()
            .filter_map(|command| command.as_str().map(String::from))
            .collect::<Vec<String>>();
        Some(command_strings)
    }
}
