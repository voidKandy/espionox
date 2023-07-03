use super::walk::{Directory, File};
use crate::agent::handler::{AgentHandler, SpecialAgent};
use serde_json::{json, Value};

pub struct Context {
    pub messages: Vec<Value>,
    pub files: Vec<File>,
    pub directories: Vec<Directory>,
    pub guidance: Option<String>,
}

impl Context {
    fn new(messages: Vec<Value>) -> Context {
        Context {
            messages,
            files: vec![],
            directories: vec![],
            guidance: None,
        }
    }
    fn make_relevant(&mut self, dirs: Option<Vec<Directory>>, files: Option<Vec<File>>) {
        match dirs {
            Some(ds) => ds.into_iter().for_each(|d| self.directories.push(d)),
            None => {}
        };
        match files {
            Some(fs) => fs.into_iter().for_each(|f| self.files.push(f)),
            None => {}
        }
    }
}

pub trait Contextual {
    fn init_context(&self, prompt: Option<&str>) -> Context;
    fn append_to_messages(&self, context: &mut Vec<Value>, role: &str, content: &str);
    fn deliver_files_to_messages(&self, context: &mut Vec<Value>, files: Vec<File>, message: &str);
}
pub trait Operations {
    fn parse_response(&self, response: Value) -> Option<Vec<String>>;
}

impl Contextual for SpecialAgent {
    fn init_context(&self, prompt: Option<&str>) -> Context {
        if let Some(prompt) = prompt {
            return Context::new(vec![(json!({"role": "system", "content": prompt}))]);
        };
        Context::new(vec![])
    }
    fn append_to_messages(&self, messages: &mut Vec<Value>, role: &str, content: &str) {
        messages.push(json!({"role": role, "content": content}))
    }
    fn deliver_files_to_messages(
        &self,
        messages: &mut Vec<Value>,
        files: Vec<File>,
        message: &str,
    ) {
        let mut payload = vec![];
        files.iter().for_each(|f| {
            payload.push(format!(
                "FilePath: {}, Content: {}",
                &f.filepath.display(),
                &f.summary
            ));
        });
        self.append_to_messages(
            messages,
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
