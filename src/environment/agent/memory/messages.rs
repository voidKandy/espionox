use super::embeddings::EmbeddingVector;
use crate::environment::agent::language_models::{embed, openai::gpt::GptMessage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub metadata: MessageMetadata,
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.role == other.role && self.content == other.content
    }
}
impl Eq for Message {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MessageRole {
    Assistant,
    User,
    System,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MetadataInfo {
    pub name: String,
    pub content: String,
    pub embedding: Option<EmbeddingVector>,
}

/// Data about a struct that can only be ascertained by a model
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MessageMetadata {
    pub content_embedding: Option<EmbeddingVector>,
    pub infos: Vec<MetadataInfo>,
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

impl Message {
    /// Best way to initialize a standard message
    pub fn new(role: MessageRole, content: &str) -> Self {
        Message {
            role,
            content: content.to_string(),
            metadata: MessageMetadata::default(),
        }
    }
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            content_embedding: None,
            infos: vec![],
        }
    }
}

impl MetadataInfo {
    pub fn new(name: &str, content: &str, create_embedding: bool) -> Self {
        let name = name.to_lowercase().to_string();
        let content = content.to_string();
        let embedding = match create_embedding {
            true => embed(&content).ok().map(|emb| emb.into()),
            false => None,
        };
        Self {
            name,
            content,
            embedding,
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

impl TryFrom<GptMessage> for FunctionMessage {
    type Error = anyhow::Error;
    fn try_from(value: GptMessage) -> Result<Self, Self::Error> {
        match value.function_call {
            Some(json_value) => Ok(FunctionMessage {
                function_call: json_value.into(),
            }),
            None => Err(anyhow::anyhow!("GptMessage doesn't contain function call")),
        }
    }
}

impl From<GptMessage> for Message {
    fn from(value: GptMessage) -> Self {
        let content = value.content.expect("Value has no content");
        Message {
            role: value.role.into(),
            content,
            metadata: MessageMetadata::default(),
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
        Message {
            role,
            content,
            metadata: MessageMetadata::default(),
        }
    }
}

impl Into<Value> for Message {
    fn into(self) -> Value {
        let role = match self.role {
            MessageRole::Other(_) => MessageRole::System.to_string(),
            other => other.to_string(),
        };

        // Model should not receive excessive whitespace or newlines
        let content = self
            .content
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .replace('\n', " ");
        json!({"role": role, "content": content})
        // Self::Function { function_call } => {
        //     let func_call_json: Value = function_call.into();
        //     json!({"role": "function", "content": null, "function_call": func_call_json})
        // }
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
