use crate::{core::*, language_models::openai::gpt::GptMessage};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Message {
    Standard {
        role: MessageRole,
        content: String,
    },
    FlatStruct {
        content: String,
        metadata: MessageMetadata,
    },
    Function {
        function_call: FunctionCall,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MessageRole {
    Assistant,
    User,
    System,
    Other(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MessageMetadata {
    File {
        path: PathBuf,
        summary: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FunctionCall {
    name: String,
    arguments: Vec<Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MessageVector(Vec<Message>);

pub trait ToMessage: std::fmt::Debug + Display + ToString + Send + Sync {
    fn to_message(&self) -> Message;
    fn to_message_with_role(&self, role: MessageRole) -> Message {
        Message::Standard {
            role,
            content: self.to_string(),
        }
    }
    fn as_file(&self) -> Option<&File> {
        None
    }
    fn as_io(&self) -> Option<&Io> {
        None
    }
    fn as_string(&self) -> Option<&String> {
        None
    }
}

impl From<&File> for MessageMetadata {
    fn from(value: &File) -> Self {
        let path = value.filepath.to_owned().into();
        let summary = value.summary.to_owned();
        MessageMetadata::File { path, summary }
    }
}

impl TryInto<Box<dyn ToMessage>> for MessageMetadata {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Box<dyn ToMessage>, Self::Error> {
        match self {
            Self::File { path, summary } => {
                let mut file = File::from(path);
                file.summary = summary;
                Ok(Box::new(file))
            }
            _ => Err(anyhow::anyhow!("No struct buildable from metadata")),
        }
    }
}

impl TryInto<Box<dyn ToMessage>> for Message {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Box<dyn ToMessage>, Self::Error> {
        if let Message::FlatStruct { metadata, .. } = self {
            if let Ok(dyn_struct) = metadata.try_into() {
                Ok(dyn_struct)
            } else {
                Err(anyhow::anyhow!("Could not build dyn struct from metadata"))
            }
        } else {
            Err(anyhow::anyhow!("Message is not of struct variety"))
        }
    }
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

impl ToMessage for String {
    fn to_message(&self) -> Message {
        Message::Standard {
            role: MessageRole::System,
            content: self.to_string(),
        }
    }
    fn as_string(&self) -> Option<&String> {
        Some(self)
    }
}

impl ToMessage for File {
    fn to_message(&self) -> Message {
        let content = match &self.summary {
            Some(sum) => sum.to_string(),
            None => {
                let first_chunk = &self.chunks[0];
                format!("The file: {} does not have a summary. Here is the first chunk: {}. Let the user know the file is not summarized if they ask about the file.", self.filepath.display().to_string(), first_chunk)
            }
        };
        let metadata = MessageMetadata::from(self);
        Message::FlatStruct { content, metadata }
    }
    fn as_file(&self) -> Option<&File> {
        Some(self)
    }
}

impl ToMessage for Io {
    fn to_message(&self) -> Message {
        Message::Standard {
            role: MessageRole::Other(String::from("io")),
            content: self.to_string(),
        }
    }
    fn as_io(&self) -> Option<&Io> {
        Some(self)
    }
}

impl Message {
    /// Best way to initialize a standard message
    pub fn new_standard(role: MessageRole, content: &str) -> Self {
        Message::Standard {
            role,
            content: content.to_string(),
        }
    }

    pub fn role(&self) -> MessageRole {
        match self {
            Self::Standard { role, .. } => role.to_owned(),
            Self::Function { .. } => MessageRole::Assistant,
            Self::FlatStruct { .. } => MessageRole::Other(String::from("struct_sharer")),
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
            output.push_str(&format!(
                "Role: [{}] Content: [{}] ",
                mess.role().to_string(),
                mess.content().unwrap()
            ));
        });
        output
    }
}

impl From<Directory> for MessageVector {
    fn from(value: Directory) -> Self {
        let mut vec = MessageVector::init();
        for file in value.files.into_iter() {
            vec.push(file.to_message());
        }
        vec
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
    pub fn push(&mut self, message: Message) {
        self.as_mut().push(message);
    }
    pub fn append(&mut self, mut messages: Self) {
        self.as_mut().append(messages.as_mut());
    }
    pub fn init() -> Self {
        MessageVector(vec![])
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clone_sans_system_prompt(&self) -> MessageVector {
        MessageVector::from(
            self.as_ref()
                .iter()
                .filter(|message| message.role() != MessageRole::System)
                .cloned()
                .collect::<Vec<Message>>(),
        )
    }
    pub fn clone_system_prompt(&self) -> MessageVector {
        MessageVector::from(
            self.as_ref()
                .iter()
                .filter(|message| message.role() == MessageRole::System)
                .cloned()
                .collect::<Vec<Message>>(),
        )
    }
    pub fn reset_to_system_prompt(&mut self) {
        self.as_mut()
            .retain(|message| message.role() == MessageRole::System);
    }
    pub fn chat_count(&self) -> usize {
        self.as_ref()
            .iter()
            .filter(|m| m.role() == MessageRole::User || m.role() == MessageRole::Assistant)
            .count()
    }

    pub fn get_structs(&self) -> Option<Vec<Box<dyn ToMessage>>> {
        let mut return_vec = Vec::new();
        for message in self.as_ref().into_iter() {
            if let Message::FlatStruct { .. } = message {
                return_vec.push(
                    message
                        .to_owned()
                        .try_into()
                        .expect("Failed to get box struct from message"),
                );
            }
        }
        if !return_vec.is_empty() {
            Some(return_vec)
        } else {
            None
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
        let role = match self.role() {
            MessageRole::Other(_) => MessageRole::System.to_string(),
            other => other.to_string(),
        };

        match self {
            Self::Standard { content, .. } => {
                // Model should not receive excessive whitespace or newlines
                let content = content
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .replace('\n', " ");
                json!({"role": role, "content": content})
            }
            Self::Function { function_call } => {
                let func_call_json: Value = function_call.into();
                json!({"role": "function", "content": null, "function_call": func_call_json})
            }
            Self::FlatStruct { content, .. } => {
                let content = content
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join(" ")
                    .replace('\n', " ");
                json!({"role": role, "content": content})
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
