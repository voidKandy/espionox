use super::{
    integrations::database::{self, EmbeddedCoreStruct, EmbeddedType},
    messages::*,
};
use crate::{
    core::{File, FileChunk},
    database::init::{DatabaseEnv, DbPool},
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, sync::Arc};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub enum Memory {
    LongTerm(String),
    #[default]
    ShortTerm,
    Forget,
}

#[derive(Debug)]
pub struct MemoryCache {
    messages: MessageVector,
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
    fn messages(&self) -> MessageVector {
        self.messages
    }

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
        static DATA_POOL: Arc<DbPool> = Arc::new(DbPool::sync_init_pool(DatabaseEnv::Default));
    }

    pub fn db_pool() -> Arc<DbPool> {
        Memory::DATA_POOL.with(|poo| Arc::clone(poo))
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
                st_mem.borrow().messages()
            );
            st_mem
                .borrow_mut()
                .messages()
                .as_mut_ref()
                .append(messages.to_owned().as_mut_ref());
            tracing::info!(
                "Cached messages after saving: {:?}",
                st_mem.borrow().messages()
            );
        });
    }

    fn get_messages_from_cache() -> MessageVector {
        Self::CACHED_MEMORY.with(|mem| {
            let st_mem = mem.borrow();
            tracing::info!("Messages loaded from Cache: {:?}", st_mem.messages());
            st_mem.messages().to_owned()
        })
    }

    pub fn load(&self) -> MessageVector {
        match self {
            Memory::LongTerm(threadname) => database::get_messages_from_database(threadname),
            Memory::ShortTerm => Self::get_messages_from_cache(),
            Memory::Forget => MessageVector::new(vec![]),
        }
    }
    pub fn save(&self, messages: MessageVector) {
        match self {
            Memory::LongTerm(threadname) => {
                database::save_messages_to_database(threadname, messages);
            }
            Memory::ShortTerm => {
                Self::save_messages_to_cache(messages);
            }
            Memory::Forget => {}
        }
    }
}
