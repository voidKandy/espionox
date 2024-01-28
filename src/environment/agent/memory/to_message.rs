use super::{messages::*, MessageVector};
use crate::{core::*, environment::Environment};
use std::fmt::Display;

pub trait ToMessage: std::fmt::Debug + Display + ToString + Send + Sync {
    fn to_message(&self) -> Message;
    fn get_metadata(&self) -> MessageMetadata {
        MessageMetadata::default()
    }
    fn role(&self) -> MessageRole;
}

pub trait ToMessageVector {
    fn to_message_vector(&self) -> MessageVector;
}

impl ToMessage for String {
    fn to_message(&self) -> Message {
        let message = Message {
            role: self.role(),
            content: self.to_string(),
            metadata: self.get_metadata(),
        };
        message
    }
    fn role(&self) -> MessageRole {
        MessageRole::System
    }
}
impl ToMessage for Io {
    fn to_message(&self) -> Message {
        let message = Message {
            role: self.role(),
            content: self.to_string(),
            metadata: self.get_metadata(),
        };
        message
    }

    fn role(&self) -> MessageRole {
        MessageRole::Other("io".to_string())
    }
}
