use super::messages::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MessageVector(Vec<Message>);

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

impl ToString for MessageVector {
    fn to_string(&self) -> String {
        let mut output = String::new();
        self.as_ref().into_iter().for_each(|mess| {
            output.push_str(&format!(
                "Role: [{}] Content: [{}] ",
                mess.role.to_string(),
                mess.content
            ));
        });
        output
    }
}

impl MessageVector {
    /// Create a new MessageVector given the content of a system prompt
    pub fn new(content: &str) -> Self {
        let message = Message::new(MessageRole::System, content);
        MessageVector::from(vec![message])
    }
    /// Push a message to the end of MessageVector
    pub fn push(&mut self, message: Message) {
        self.as_mut().push(message);
    }
    /// Append another MessageVector to the end of this one
    pub fn append(&mut self, mut messages: Self) {
        self.as_mut().append(messages.as_mut());
    }
    /// Create empty MessageVector
    pub fn init() -> Self {
        MessageVector(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Clone the MessageVector & remove any messages with system role
    pub fn clone_sans_system_prompt(&self) -> MessageVector {
        MessageVector::from(
            self.as_ref()
                .iter()
                .filter(|message| message.role != MessageRole::System)
                .cloned()
                .collect::<Vec<Message>>(),
        )
    }

    /// Clone the MessageVector & remove any messages without system role
    pub fn clone_system_prompt(&self) -> MessageVector {
        MessageVector::from(
            self.as_ref()
                .iter()
                .filter(|message| message.role == MessageRole::System)
                .cloned()
                .collect::<Vec<Message>>(),
        )
    }

    /// Remove any messages without system role
    pub fn reset_to_system_prompt(&mut self) {
        self.as_mut()
            .retain(|message| message.role == MessageRole::System);
    }

    /// Count the amount of User/Assistant messages
    pub fn chat_count(&self) -> usize {
        self.as_ref()
            .iter()
            .filter(|m| m.role == MessageRole::User || m.role == MessageRole::Assistant)
            .count()
    }

    // pub fn get_structs(&self) -> Option<Vec<Box<dyn ToMessage>>> {
    //     let mut return_vec = Vec::new();
    //     for message in self.as_ref().into_iter() {
    //         if let Message::FlatStruct { .. } = message {
    //             return_vec.push(
    //                 message
    //                     .to_owned()
    //                     .try_into()
    //                     .expect("Failed to get box struct from message"),
    //             );
    //         }
    //     }
    //     if !return_vec.is_empty() {
    //         Some(return_vec)
    //     } else {
    //         None
    //     }
    // }
}
