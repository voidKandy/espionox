use super::walk::File;
use crate::agent::handler::SpecialAgent;
use serde_json::{json, Value};

pub trait Context {
    fn init_context(&self, prompt: Option<&str>) -> Vec<Value>;
    fn append_message(&self, context: &mut Vec<Value>, role: &str, content: &str);
    fn append_files(&self, context: &mut Vec<Value>, files: Vec<File>, message: &str);
}

impl Context for SpecialAgent {
    fn init_context(&self, prompt: Option<&str>) -> Vec<Value> {
        let mut new_context = vec![];
        match prompt {
            Some(prompt) => self.append_message(&mut new_context, "system", prompt),
            None => self.append_message(&mut new_context, "system", &self.get_sys_prompt()),
        }
        new_context
    }
    fn append_message(&self, context: &mut Vec<Value>, role: &str, content: &str) {
        context.push(json!({"role": role, "content": content}))
    }
    fn append_files(&self, context: &mut Vec<Value>, files: Vec<File>, message: &str) {
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
