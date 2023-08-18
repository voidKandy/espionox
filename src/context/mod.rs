pub mod memory;
pub mod messages;

pub use memory::*;
pub use messages::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Context {
    pub memory: Memory,
    pub buffer: MessageVector,
}

impl Context {
    pub fn build(memory: Memory) -> Context {
        Context {
            buffer: memory.load(),
            memory,
        }
    }

    pub fn info_display(&self) -> String {
        let buffer = self.buffer_as_string();
        let current_mem = match &self.memory {
            Memory::Forget => "Forget".to_string(),
            Memory::ShortTerm => "ShortTerm".to_string(),
            Memory::LongTerm(thread) => {
                format!("LongTermThread: {}", thread.clone())
            }
        };
        format!("In {current_mem}\n\nBuffer:\n{buffer}")
    }

    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer
            .as_mut_ref()
            .push(Message::new(role.to_string(), content.to_string()));
    }

    pub fn buffer_as_string(&self) -> String {
        let mut output = String::new();
        self.buffer.as_ref().into_iter().for_each(|mess| {
            output.push_str(&format!("{}\n", mess));
        });
        format!("{}", output)
    }

    pub fn save_buffer(&self) {
        let buf_difference = MessageVector::new(
            self.buffer
                .as_ref()
                .iter()
                .filter(|&value| !self.memory.load().as_ref().contains(value))
                .cloned()
                .collect(),
        );
        self.memory.save(buf_difference);
    }
}
