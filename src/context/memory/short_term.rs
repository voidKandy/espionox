#[cfg(feature = "long_term_memory")]
use super::super::integrations::database::{EmbeddedCoreStruct, EmbeddedType};

use super::{MemoryVariant, MessageVector};
// use crate::core::{File, FileChunk};

#[derive(Debug, Clone, PartialEq)]
pub struct ShortTermMemory {
    pub(super) cache: MessageVector,
    recall: RecallMode,
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        ShortTermMemory {
            cache: MessageVector::init(),
            recall: RecallMode::default(),
        }
    }
}

impl ShortTermMemory {
    pub fn build_empty(recall: RecallMode) -> Self {
        Self {
            cache: MessageVector::init(),
            recall,
        }
    }

    pub fn build_with_prompt(recall: RecallMode, init_prompt: MessageVector) -> Self {
        Self {
            cache: init_prompt,
            recall,
        }
    }

    pub fn save(&mut self, mut messages: MessageVector) {
        match self.recall {
            RecallMode::Forgetful => {}
            _ => {
                self.cache.as_mut().append(messages.as_mut());
            }
        }
    }

    pub fn reset(mut self) {
        let recall = self.recall;
        self = ShortTermMemory {
            cache: MessageVector::init(),
            recall,
        };
    }

    // pub fn load(&self) {
    //     match self.recall {
    //         RecallMode::Default => {}
    //     }
    // }
}

// impl Memory for ShortTermMemory {
//     fn load(&self) -> MessageVector {
//         match self {
//             Self::Cache(mem) => mem.messages.to_owned(),
//             Self::Forget => MessageVector::init(),
//         }
//     }
//
//     #[tracing::instrument(name = "Save messages to ShortTerm")]
//     fn save(&mut self, messages: MessageVector) {
//         match self {
//             Self::Cache(mem) => {
//                 tracing::info!("Cached messages before saving: {:?}", mem.messages);
//                 mem.messages.as_mut().append(messages.to_owned().as_mut());
//                 tracing::info!("Cached messages after saving: {:?}", mem.messages);
//             }
//             Self::Forget => {}
//         }
//     }
// }

// impl ToString for ShortTermMemory {
//     fn to_string(&self) -> String {
//         match self {
//             Self::Cache(_) => "Cached".to_string(),
//             Self::Forget => "Forget".to_string(),
//         }
//     }
// }

// impl MemoryCache {
// #[cfg(feature = "long_term_memory")]
// pub fn save_embedded_to_cache(&mut self, embedded: EmbeddedCoreStruct) {
//     match embedded.get_type() {
//         &EmbeddedType::File => self.push_file(embedded.try_as_file().unwrap()),
//         &EmbeddedType::Chunk => self.push_chunk(embedded.try_as_chunk().unwrap()),
//     }
// }
// }

// impl ShortTermMemory {
//     pub fn new_cache() -> Self {
//         ShortTermMemory::Cache(MemoryCache::default())
//     }
// }
