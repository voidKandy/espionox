use super::{super::config::memory::Memory, memory::LoadedMemory};
use crate::{core::file_interface::File, database::api};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Context {
    pub memory: Memory,
    pub buffer: Vec<Value>,
}

impl Context {
    pub fn build(memory: Memory) -> Context {
        Context {
            buffer: memory.load(),
            memory,
        }
    }

    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer.push(json!({"role": role, "content": content}));
    }
    pub fn remember_file(&self, filepath: &str) {
        let file = File::build(filepath);
        match &self.memory {
            Memory::Remember(LoadedMemory::LongTerm(threadname)) => {
                let sql_tup = api::sql_from_file(file, threadname);
                LoadedMemory::LongTerm(threadname.to_string()).store_file_tup(sql_tup);
            }
            _ => panic!("Memory not long term"),
        };
    }
}
