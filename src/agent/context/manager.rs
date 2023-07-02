//json!({ "model": "gpt-3.5-turbo", "messages": [{"role": "system", "content": self.config.system_message}, {"role": "user", "content": prompt}], "max_tokens": 2000, "n": 1, "stop": null }))
use super::*;
use crate::agent::handler::SpecialAgent;
use serde_json::{json, Value};
use std::error::Error;

pub trait Context {
    fn init_context(&self) -> Vec<Value>;
    fn append_message(&self, context: &mut Vec<Value>, role: &str, content: &str);
}
impl Context for SpecialAgent {
    fn init_context(&self) -> Vec<Value> {
        let mut new_context = vec![];
        self.append_message(&mut new_context, "system", &self.get_sys_prompt());
        new_context
    }
    fn append_message(&self, context: &mut Vec<Value>, role: &str, content: &str) {
        context.push(json!({"role": role, "content": content}))
    }
}

// {"role": "user", "content": prompt}
