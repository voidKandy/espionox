use std::{cmp::Ordering, option::IterMut, vec::IntoIter};

use super::messages::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct MessageStack(pub(crate) Vec<Message>);

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MessageStackRef<'stack>(pub(crate) Vec<&'stack Message>);

impl<'stack> From<Vec<&'stack Message>> for MessageStackRef<'stack> {
    fn from(value: Vec<&'stack Message>) -> Self {
        Self(value)
    }
}

impl TryFrom<Vec<Value>> for MessageStack {
    type Error = anyhow::Error;
    fn try_from(json_vec: Vec<Value>) -> Result<Self, Self::Error> {
        let mut vec: Vec<Message> = vec![];
        for val in json_vec.into_iter() {
            let m = Message::try_from(val)?;
            vec.push(m);
        }
        Ok(Self(vec))
    }
}

impl IntoIterator for MessageStack {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Into<MessageStack> for MessageStackRef<'_> {
    fn into(self) -> MessageStack {
        MessageStack(self.0.into_iter().map(|m| m.clone()).collect())
    }
}

impl From<Vec<Message>> for MessageStack {
    fn from(value: Vec<Message>) -> Self {
        let (all_system, mut rest): (Vec<Message>, Vec<Message>) = value
            .into_iter()
            .partition(|m| m.role.actual() == &MessageRole::System);
        let sys_message =
            all_system
                .into_iter()
                .enumerate()
                .fold(Message::new_system(""), |mut mess, (i, m)| {
                    let content = if i > 0 {
                        &format!(" {}", m.content)
                    } else {
                        &m.content
                    };
                    mess.content.push_str(content);
                    mess
                });
        if !sys_message.content.is_empty() {
            rest.reverse();
            rest.push(sys_message);
            rest.reverse();
        }
        Self(rest)
    }
}

impl AsRef<Vec<Message>> for MessageStack {
    fn as_ref(&self) -> &Vec<Message> {
        &self.0
    }
}

impl AsMut<Vec<Message>> for MessageStack {
    fn as_mut(&mut self) -> &mut Vec<Message> {
        &mut self.0
    }
}

impl ToString for MessageStack {
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

impl<'stack> MessageStack {
    /// Create empty MessageStack
    pub fn init() -> Self {
        MessageStack(vec![])
    }

    /// Create a new MessageStack given the content of a system prompt
    /// will panic if passed an empty string
    pub fn new(content: &str) -> Self {
        if content.is_empty() {
            panic!("cannot create message with empty content to message stack");
        }
        let message = Message::new_system(content);
        MessageStack::from(vec![message])
    }

    /// Returns mutable access to the content of the system prompt of the agent
    pub fn mut_system_prompt_content(&mut self) -> Option<&mut String> {
        self.0.first_mut().and_then(|m| {
            if m.role.actual() == &MessageRole::System {
                Some(&mut m.content)
            } else {
                None
            }
        })
    }

    pub fn ref_system_prompt_content(&self) -> Option<&str> {
        self.0.first().and_then(|m| {
            if m.role.actual() == &MessageRole::System {
                Some(m.content.as_str())
            } else {
                None
            }
        })
    }

    // pub fn mutate_system_prompt(&mut self, f: impl FnOnce(&mut String)) {
    //     if let Some(first_message) = self.0.first_mut() {
    //         f(&mut first_message.content);
    //     }
    // }

    /// Push a message to the end of MessageStack, does nothing if the message's content is empty
    pub fn push(&mut self, message: Message) {
        if &MessageRole::System == message.role.actual() && self.len() > 0 {
            if let Some(sys_prompt) = self.mut_system_prompt_content() {
                sys_prompt.push_str(&format!(" {}", message.content))
            }
        } else {
            if message.content.is_empty() {
                warn!("cannot push message with empty content to message stack");
                return;
            }
            self.as_mut().push(message);
        }

        if self.ref_filter_by(&MessageRole::System, true).len() > 1 {
            panic!(
                "expected to get <= 1 system prompts, got {}",
                self.ref_filter_by(&MessageRole::System, true).len()
            )
        }
    }

    /// Append another MessageStack to the end of this one
    pub fn append(&mut self, mut messages: Self) {
        self.as_mut().append(messages.as_mut());
    }

    /// Pop the last Message off the stack
    pub fn pop(&mut self, role: Option<MessageRole>) -> Option<Message> {
        if let Some(role) = role {
            for i in (0..self.len()).rev() {
                if self.0[i].role == role {
                    return Some(self.0.remove(i));
                }
            }
            return None;
        }
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Mutates message vector in place. Excludes/Explicitly includes given message role
    pub fn mut_filter_by(&mut self, role: &MessageRole, inclusive: bool) {
        match inclusive {
            true => self.0.retain(|m| &m.role == role),
            false => self.0.retain(|m| &m.role != role),
        }
    }

    /// Returns a MessageStackRef of self. Excludes/Explicitly includes given message role
    pub fn ref_filter_by(
        &'stack self,
        role: &MessageRole,
        inclusive: bool,
    ) -> MessageStackRef<'stack> {
        match inclusive {
            true => self
                .0
                .iter()
                .filter(|m| &m.role == role)
                .collect::<Vec<&'stack Message>>()
                .into(),
            false => self
                .0
                .iter()
                .filter(|m| &m.role != role)
                .collect::<Vec<&'stack Message>>()
                .into(),
        }
    }
}

impl<'stack> MessageStackRef<'stack> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self, role: Option<MessageRole>) -> Option<&'stack Message> {
        if let Some(role) = role {
            for i in (0..self.len()).rev() {
                if self.0[i].role == role {
                    return Some(self.0.remove(i));
                }
            }

            return None;
        }
        self.0.pop()
    }

    /// Same effect as `filter_by` on MessageStack, except it consumes `MessageStackRef`
    pub fn filter_by(self, role: &MessageRole, inclusive: bool) -> MessageStackRef<'stack> {
        match inclusive {
            true => self
                .0
                .into_iter()
                .filter(|m| &m.role == role)
                .collect::<Vec<&'stack Message>>()
                .into(),
            false => self
                .0
                .into_iter()
                .filter(|m| &m.role != role)
                .collect::<Vec<&'stack Message>>()
                .into(),
        }
    }
}

mod tests {
    use super::{Message, MessageStack};

    #[test]
    fn message_from_correct() {
        let messages = vec![
            Message::new_system("System"),
            Message::new_user("User message"),
            Message::new_assistant("Assistant message"),
            Message::new_user("User message"),
            Message::new_system("System"),
            Message::new_system("System"),
        ];
        let mut stack = MessageStack::from(messages);
        stack.push(Message::new_system("End system"));

        assert_eq!(
            stack.ref_system_prompt_content().unwrap(),
            "System System System End system"
        )
    }
}
