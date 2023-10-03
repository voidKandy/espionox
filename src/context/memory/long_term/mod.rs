#[cfg(feature = "long_term_memory")]
pub mod feature;

#[cfg(feature = "long_term_memory")]
use feature::{DbPool, MemoryThread};

use crate::context::{Memory, MessageVector};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LongTermMemory {
    #[default]
    None,
    #[cfg(feature = "long_term_memory")]
    Some(MemoryThread),
}

impl Memory for LongTermMemory {
    fn save(&mut self, _messages: MessageVector) {
        match self {
            Self::None => {
                tracing::warn!(
                    "Memory just saved to empty long term memory, an error may have been made. Consider using ShortTermMemory::Forget"
                );
            }
            #[cfg(feature = "long_term_memory")]
            Self::Some(mem) => {
                mem.save_messages_to_database(_messages);
            }
        }
    }

    fn load(&self) -> MessageVector {
        match self {
            Self::None => {
                tracing::warn!(
                    "Memory just loaded from empty long term memory, an error may have been made. Consider using ShortTermMemory::Forget"
                );

                MessageVector::init()
            }

            #[cfg(feature = "long_term_memory")]
            Self::Some(mem) => mem.get_messages_from_database(),
        }
    }
}

impl ToString for LongTermMemory {
    fn to_string(&self) -> String {
        match self {
            Self::None => "None".to_string(),
            #[cfg(feature = "long_term_memory")]
            Self::Some(mem) => mem.threadname().to_owned().expect("No threadname"),
        }
    }
}

#[cfg(feature = "long_term_memory")]
impl LongTermMemory {
    pub fn try_pool(&self) -> Option<DbPool> {
        match self {
            LongTermMemory::Some(ltm) => Some(ltm.pool()),
            _ => None,
        }
    }
}
