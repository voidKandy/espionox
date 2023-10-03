#[cfg(feature = "long_term_memory")]
use super::super::integrations::database::{EmbeddedCoreStruct, EmbeddedType};

use super::{Memory, MessageVector};
use crate::core::{File, FileChunk};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ShortTermMemory {
    Cache(MemoryCache),
    #[default]
    Forget,
}

#[derive(Debug, Clone, PartialEq)]
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

#[allow(unused)]
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
        match self {
            Self::Cache(mem) => mem.messages.to_owned(),
            Self::Forget => MessageVector::init(),
        }
    }

    #[tracing::instrument(name = "Save messages to ShortTerm")]
    fn save(&mut self, messages: MessageVector) {
        match self {
            Self::Cache(mem) => {
                tracing::info!("Cached messages before saving: {:?}", mem.messages);
                mem.messages.as_mut().append(messages.to_owned().as_mut());
                tracing::info!("Cached messages after saving: {:?}", mem.messages);
            }
            Self::Forget => {}
        }
    }
}

impl ToString for ShortTermMemory {
    fn to_string(&self) -> String {
        match self {
            Self::Cache(_) => "Cached".to_string(),
            Self::Forget => "Forget".to_string(),
        }
    }
}

impl MemoryCache {
    #[cfg(feature = "long_term_memory")]
    pub fn save_embedded_to_cache(&mut self, embedded: EmbeddedCoreStruct) {
        match embedded.get_type() {
            &EmbeddedType::File => self.push_file(embedded.try_as_file().unwrap()),
            &EmbeddedType::Chunk => self.push_chunk(embedded.try_as_chunk().unwrap()),
        }
    }
}

impl ShortTermMemory {
    pub fn new_cache() -> Self {
        ShortTermMemory::Cache(MemoryCache::default())
    }
}
