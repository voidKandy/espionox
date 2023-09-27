use super::{
    super::integrations::database::{EmbeddedCoreStruct, EmbeddedType},
    Memory, MessageVector,
};
use crate::core::{File, FileChunk};
use std::cell::RefCell;

#[derive(Debug, Clone, Default)]
pub struct ShortTermMemory {}

#[derive(Debug)]
pub struct MemoryCache {
    pub messages: MessageVector,
    embedded_files: Vec<File>,
    embedded_chunks: Vec<FileChunk>,
}

impl Default for MemoryCache {
    fn default() -> Self {
        MemoryCache {
            messages: MessageVector::init(),
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

impl Memory for ShortTermMemory {
    fn load(&self) -> MessageVector {
        Self::CACHED_MEMORY.with(|mem| {
            let st_mem = mem.borrow();
            tracing::info!("Messages loaded from Cache: {:?}", st_mem.messages);
            st_mem.messages.to_owned()
        })
    }

    #[tracing::instrument(name = "Save messages to cached static MessageVector")]
    fn save(&self, messages: MessageVector) {
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
}

impl ShortTermMemory {
    thread_local! {
        static CACHED_MEMORY: RefCell<MemoryCache> = RefCell::new(MemoryCache::default());
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
}
