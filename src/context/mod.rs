pub mod integrations;
pub mod memory;
pub mod messages;

pub use memory::*;
pub use messages::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
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

    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer
            .as_mut_ref()
            .push(Message::new_standard(role, content));
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
