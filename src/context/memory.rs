use super::{
    integrations::database::{self, EmbeddedCoreStruct, EmbeddedType},
    messages::*,
};
use crate::{
    core::{File, FileChunk},
    database::DbPool,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum Memory {
    LongTerm(String),
    #[default]
    ShortTerm,
    Forget,
}

#[derive(Debug)]
pub struct MemoryCache {
    pub messages: MessageVector,
    embedded_files: Vec<File>,
    embedded_chunks: Vec<FileChunk>,
}

impl Default for MemoryCache {
    fn default() -> Self {
        MemoryCache {
            messages: MessageVector::new(Vec::new()),
            embedded_files: vec![],
            embedded_chunks: vec![],
        }
    }
}

impl MemoryCache {
    fn push_file(&mut self, file: File) {
        self.embedded_files.push(file);
    }

    fn push_chunk(&mut self, chunk: FileChunk) {
        self.embedded_chunks.push(chunk);
    }
}

impl Memory {
    thread_local! {
        static CACHED_MEMORY: RefCell<MemoryCache> = RefCell::new(MemoryCache::default());
    }

    pub fn threadname(&self) -> Option<String> {
        match self {
            Self::LongTerm(threadname) => Some(threadname.to_string()),
            _ => None,
        }
    }

    pub fn save_embedded_to_cache(embedded: EmbeddedCoreStruct) {
        Self::CACHED_MEMORY.with(|st_mem| match embedded.get_type() {
            EmbeddedType::File => st_mem
                .borrow_mut()
                .push_file(embedded.try_as_file().unwrap()),
            EmbeddedType::Chunk => st_mem
                .borrow_mut()
                .push_chunk(embedded.try_as_chunk().unwrap()),
        })
    }

    #[tracing::instrument(name = "Save messages to cached static MessageVector")]
    fn save_messages_to_cache(messages: MessageVector) {
        Self::CACHED_MEMORY.with(|st_mem| {
            tracing::info!(
                "Cached messages before saving: {:?}",
                st_mem.borrow().messages
            );
            st_mem
                .borrow_mut()
                .messages
                .as_mut()
                .append(messages.to_owned().as_mut());
            tracing::info!(
                "Cached messages after saving: {:?}",
                st_mem.borrow().messages
            );
        });
    }

    fn get_messages_from_cache() -> MessageVector {
        Self::CACHED_MEMORY.with(|mem| {
            let st_mem = mem.borrow();
            tracing::info!("Messages loaded from Cache: {:?}", st_mem.messages);
            st_mem.messages.to_owned()
        })
    }

    pub fn load(&self, pool: Option<&DbPool>) -> MessageVector {
        match self {
            Memory::LongTerm(threadname) => {
                let pool = pool.unwrap();
                database::get_messages_from_database(threadname, &pool)
            }
            Memory::ShortTerm => Self::get_messages_from_cache(),
            Memory::Forget => MessageVector::new(vec![]),
        }
    }
    pub fn save(&self, messages: MessageVector, pool: Option<&DbPool>) {
        match self {
            Memory::LongTerm(threadname) => {
                let pool = pool.unwrap();
                database::save_messages_to_database(threadname, messages, &pool);
            }
            Memory::ShortTerm => {
                Self::save_messages_to_cache(messages);
            }
            Memory::Forget => {}
        }
    }
}
