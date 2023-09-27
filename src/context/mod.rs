pub mod integrations;
pub mod memory;
pub mod messages;

pub use memory::*;
pub use messages::*;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::configuration::ConfigEnv;

use self::short_term::ShortTermMemory;

#[cfg(feature = "long_term_memory")]
use self::long_term::{database::DbPool, LongTermMemory};

#[derive(Clone, Debug)]
pub enum MemoryVariant {
    Short(ShortTermMemory),
    Forget,
    #[cfg(feature = "long_term_memory")]
    Long(LongTermMemory),
}

impl Default for MemoryVariant {
    fn default() -> Self {
        Self::Short(ShortTermMemory::default())
    }
}

impl ToString for MemoryVariant {
    fn to_string(&self) -> String {
        match self {
            MemoryVariant::Forget => "Forget".to_string(),
            MemoryVariant::Short(_) => "ShortTerm".to_string(),
            #[cfg(feature = "long_term_memory")]
            MemoryVariant::Long(m) => {
                format!("LongTerm Thread: {:?}", m.threadname())
            }
        }
    }
}

impl MemoryVariant {
    pub fn new_short() -> Self {
        Self::Short(ShortTermMemory::default())
    }

    #[cfg(feature = "long_term_memory")]
    pub fn new_long(pool: Arc<DbPool>) -> Self {
        Self::Long(LongTermMemory::init(pool))
    }

    #[cfg(feature = "long_term_memory")]
    pub fn long_term_memory(&self) -> LongTermMemory {
        match self {
            Self::Long(mem) => mem.to_owned(),
            _ => panic!("Do not use this function unless MemoryVariant is Long"),
        }
    }

    fn load(&self) -> MessageVector {
        match self {
            MemoryVariant::Short(mem) => mem.load(),
            MemoryVariant::Forget => MessageVector::init(),
            #[cfg(feature = "long_term_memory")]
            MemoryVariant::Long(mem) => mem.load(),
        }
    }

    fn save(&self, messages: MessageVector) {
        match self {
            MemoryVariant::Short(mem) => mem.save(messages),
            MemoryVariant::Forget => {}
            #[cfg(feature = "long_term_memory")]
            MemoryVariant::Long(mem) => mem.save(messages),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Context {
    #[serde(skip_serializing, skip_deserializing)]
    pub memory: MemoryVariant,
    pub buffer: MessageVector,
    #[serde(skip_serializing, skip_deserializing)]
    pub env: ConfigEnv,
}

impl Context {
    pub fn build(memory: MemoryVariant, env: ConfigEnv) -> Context {
        Context {
            buffer: memory.load(),
            memory,
            env,
        }
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
