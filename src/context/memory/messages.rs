use crate::{core::*, language_models::openai::gpt::GptMessage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::{self, Display};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    Standard { role: MessageRole, content: String },
    Function { function_call: FunctionCall },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MessageRole {
    Assistant,
    User,
    System,
    Other(String),
}

impl ToString for MessageRole {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Other(other) => other,
        })
    }
}

impl From<String> for MessageRole {
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        match value.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            other => MessageRole::Other(other.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionCall {
    name: String,
    arguments: Vec<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MessageVector(Vec<Message>);

pub trait ToMessage: std::fmt::Debug + Display + ToString {
    fn to_message(&self, role: MessageRole) -> Message {
        Message::new_standard(role, &format!("{}", self))
    }
}

impl ToMessage for String {}
impl ToMessage for str {}

impl ToMessage for FileChunk {}
impl ToMessage for File {}
impl ToMessage for Directory {}
impl ToMessage for Io {}

impl Message {
    pub fn new_standard(role: MessageRole, content: &str) -> Self {
        let content = content.to_string();
        Message::Standard { role, content }
    }

    pub fn role(&self) -> MessageRole {
        match self {
            Self::Standard { role, .. } => role.to_owned(),
            Self::Function { .. } => MessageRole::Assistant,
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

impl Into<Vec<Value>> for &MessageVector {
    fn into(self) -> Vec<Value> {
        self.0
            .to_owned()
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<Value>>()
    }
}

impl MessageVector {
    pub fn from_message(message: Message) -> Self {
        MessageVector::from(vec![message])
    }
    pub fn init() -> Self {
        MessageVector(vec![])
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn reset_to_system_prompt(&mut self) {
        self.as_mut()
            .retain(|message| message.role() == MessageRole::System);
    }
    pub fn len_excluding_system_prompt(&self) -> usize {
        self.as_ref()
            .iter()
            .filter(|m| m.role() != MessageRole::System)
            .count()
    }
    pub fn push(&mut self, message: Message) {
        self.as_mut().push(message);
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
                    role: value.role.into(),
                    content,
                }
            }
        }
    }
}

impl From<Value> for Message {
    fn from(json: Value) -> Self {
        let role = json
            .get("role")
            .expect("Couldn't get role")
            .to_string()
            .into();
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
            self.role().to_string(),
            self.content()
        )
    }
}
