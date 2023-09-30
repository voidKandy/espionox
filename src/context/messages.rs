use crate::language_models::openai::gpt::GptMessage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    Standard { role: String, content: String },
    Function { function_call: FunctionCall },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionCall {
    name: String,
    arguments: Vec<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MessageVector(Vec<Message>);

impl Message {
    pub fn new_standard(role: &str, content: &str) -> Self {
        let role = role.to_string();
        let content = content.to_string();
        Message::Standard { role, content }
    }

    pub fn role(&self) -> String {
        match self {
            Self::Standard { role, .. } => role.to_owned(),
            Self::Function { .. } => "assistant".to_string(),
        }
    }

    pub fn content(&self) -> Option<String> {
        match self {
            Self::Standard { content, .. } => Some(content.to_owned()),
            _ => None,
        }
    }
}

impl ToString for MessageVector {
    fn to_string(&self) -> String {
        let mut output = String::new();
        self.as_ref().into_iter().for_each(|mess| {
            output.push_str(&format!("{}\n", mess));
        });
        format!("{}", output)
    }
}

impl From<Vec<Message>> for MessageVector {
    fn from(value: Vec<Message>) -> Self {
        Self(value)
    }
}

impl MessageVector {
    pub fn new(message: Message) -> Self {
        MessageVector::from(vec![message])
    }
    pub fn init_with_system_prompt(system_prompt: &str) -> Self {
        MessageVector::from(vec![Message::new_standard("system", system_prompt)])
    }
    pub fn init() -> Self {
        MessageVector(vec![])
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn push_std(&mut self, role: &str, content: &str) {
        self.as_mut().push(Message::new_standard(role, content));
    }
}

impl AsRef<Vec<Message>> for MessageVector {
    fn as_ref(&self) -> &Vec<Message> {
        &self.0
    }
}

impl AsMut<Vec<Message>> for MessageVector {
    fn as_mut(&mut self) -> &mut Vec<Message> {
        &mut self.0
    }
}

impl Into<Vec<Value>> for MessageVector {
    fn into(self) -> Vec<Value> {
        self.0.into_iter().map(|m| m.into()).collect::<Vec<Value>>()
    }
}

impl From<Value> for FunctionCall {
    fn from(value: Value) -> Self {
        let name = value.get("name").expect("Failed to get name").to_string();
        let arguments = value
            .get("arguments")
            .expect("Failed to get args")
            .as_array()
            .expect("Failed to get arguments array")
            .to_vec();
        Self { name, arguments }
    }
}

impl Into<Value> for FunctionCall {
    fn into(self) -> Value {
        json!({"name": self.name, "arguments": self.arguments})
    }
}

impl From<GptMessage> for Message {
    fn from(value: GptMessage) -> Self {
        match value.function_call {
            Some(json_value) => Message::Function {
                function_call: json_value.into(),
            },
            None => {
                let content = value.content.expect("Value has no content");
                Message::Standard {
                    role: value.role,
                    content,
                }
            }
        }
    }
}

impl From<Value> for Message {
    fn from(json: Value) -> Self {
        let role = json.get("role").expect("Couldn't get role").to_string();
        let content = json
            .get("content")
            .expect("Couldn't get content")
            .to_string();
        Message::Standard { role, content }
    }
}

impl Into<Value> for Message {
    fn into(self) -> Value {
        match self {
            Self::Standard { role, content } => json!({"role": role, "content": content}),
            Self::Function { function_call } => {
                let func_call_json: Value = function_call.into();
                json!({"role": "assistant", "content": null, "function_call": func_call_json})
            }
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nRole: {}\nContent: {:?}\n",
            self.role(),
            self.content()
        )
    }
}
