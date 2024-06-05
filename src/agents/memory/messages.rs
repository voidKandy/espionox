use super::MessageStack;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role && self.content == other.content
    }
}
impl Eq for Message {}

pub trait ToMessage: std::fmt::Debug + Send + Sync {
    fn to_message(&self, role: MessageRole) -> Message;
}

pub trait ToMessageStack {
    fn to_message_stack(&self) -> MessageStack;
}

impl ToMessage for String {
    fn to_message(&self, role: MessageRole) -> Message {
        Message {
            role,
            content: self.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum OtherRoleTo {
    Assistant,
    User,
    System,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MessageRole {
    Assistant,
    User,
    System,
    Other {
        alias: String,
        coerce_to: OtherRoleTo,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionMessage {
    pub function_call: FunctionCall,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionCall {
    name: String,
    arguments: Vec<Value>,
}

impl ToString for MessageRole {
    fn to_string(&self) -> String {
        match self.actual() {
            &Self::System => String::from("system"),
            &Self::User => String::from("user"),
            &Self::Assistant => String::from("assistant"),
            _ => unreachable!(),
        }
    }
}

impl TryFrom<String> for MessageRole {
    type Error = anyhow::Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let value = string.to_lowercase();
        match value.as_str() {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            e => Err(anyhow!("Cannot coerce string: [{}] to MessageRole", e)),
        }
    }
}

impl MessageRole {
    /// Returns `actual` role of message. Either User, Assistant or System
    pub fn actual(&self) -> &Self {
        if let MessageRole::Other { coerce_to, .. } = &self {
            return match coerce_to {
                OtherRoleTo::User => &MessageRole::User,
                OtherRoleTo::System => &MessageRole::System,
                OtherRoleTo::Assistant => &MessageRole::Assistant,
            };
        }
        &self
    }
}

impl Message {
    pub fn new_other(alias: &str, content: &str, coerce_to: OtherRoleTo) -> Self {
        Message {
            role: MessageRole::Other {
                alias: alias.to_owned(),
                coerce_to,
            },
            content: content.to_string(),
        }
    }

    pub fn new_system(content: &str) -> Self {
        Message {
            role: MessageRole::System,
            content: content.to_string(),
        }
    }

    pub fn new_user(content: &str) -> Self {
        Message {
            role: MessageRole::User,
            content: content.to_string(),
        }
    }

    pub fn new_assistant(content: &str) -> Self {
        Message {
            role: MessageRole::Assistant,
            content: content.to_string(),
        }
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

impl TryFrom<Value> for Message {
    type Error = anyhow::Error;
    fn try_from(json: Value) -> Result<Self, Self::Error> {
        let role = json
            .get("role")
            .expect("Couldn't get role")
            .to_string()
            .replace('"', "")
            .try_into()?;
        let content = json
            .get("content")
            .expect("Couldn't get content")
            .to_string();
        Ok(Message { role, content })
    }
}

impl Into<Value> for Message {
    fn into(self) -> Value {
        let content = self
            .content
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .replace('\n', " ");
        json!({"role": self.role.to_string(), "content": content})
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\nRole: {}\nContent: {:?}\n",
            self.role.to_string(),
            self.content
        )
    }
}
