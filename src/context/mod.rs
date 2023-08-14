pub mod memory;
use memory::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Context {
    pub memory: Memory,
    buffer: Vec<Value>,
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

    pub fn buf_ref(&self) -> Vec<Value> {
        self.buffer.clone()
    }

    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer.push(json!({"role": role, "content": content}));
    }

    pub fn buffer_as_string(&self) -> String {
        let mut output = String::new();
        self.buf_ref().into_iter().for_each(|obj| {
            if let Some(role) = obj.get("role") {
                if let Some(content) = obj.get("content") {
                    output.push_str(&format!("\nRole: {}\nContent: {}\n\n", role, content));
                }
            }
        });
        output
    }

    pub fn save_buffer(&self) {
        let buf_difference = self
            .buf_ref()
            .iter()
            .filter(|&value| !self.memory.load().contains(value))
            .cloned()
            .collect();
        self.memory.save(buf_difference);
    }
}
