pub mod memory;
pub mod messages;

use memory::*;
use messages::*;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
            Memory::Remember(LoadedMemory::Cache) => "Cache".to_string(),
            Memory::Remember(LoadedMemory::LongTerm(thread)) => {
                format!("LongTermThread: {}", thread.clone())
            }
        };
        format!("In {current_mem}\n\nBuffer:\n{buffer}")
    }
    //
    // pub fn buf_ref(&self) -> MessageVector {
    //     self.buffer.clone()
    // }

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
        format!("Buffer: {}", output)
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
