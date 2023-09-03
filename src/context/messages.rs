use crate::database::models::messages::MessageModelSql;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MessageVector(Vec<Message>);

impl Message {
    pub fn new(role: &str, content: &str) -> Self {
        let role = role.to_string();
        let content = content.to_string();
        Message { role, content }
    }
    pub fn role(&self) -> &String {
        &self.role
    }

    pub fn content(&self) -> &String {
        &self.content
    }
}

impl MessageVector {
    pub fn new(messages: Vec<Message>) -> Self {
        MessageVector(messages)
    }
    pub fn as_mut_ref(&mut self) -> &mut Vec<Message> {
        &mut self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl AsRef<Vec<Message>> for MessageVector {
    fn as_ref(&self) -> &Vec<Message> {
        &self.0
    }
}

impl Into<Vec<Value>> for MessageVector {
    fn into(self) -> Vec<Value> {
        self.0.into_iter().map(|m| m.into()).collect::<Vec<Value>>()
    }
}

impl From<Value> for Message {
    fn from(json: Value) -> Self {
        let role = json.get("role").expect("Couldn't get role").to_string();
        let content = json
            .get("content")
            .expect("Couldn't get content")
            .to_string();
        Message { role, content }
    }
}

impl Into<Value> for Message {
    fn into(self) -> Value {
        json!({"role": self.role, "content": self.content})
    }
}

impl From<MessageModelSql> for Message {
    fn from(sql_model: MessageModelSql) -> Self {
        Message {
            role: sql_model.role,
            content: sql_model.content,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\nRole: {}\nContent: {}\n", self.role, self.content)
    }
}
