#[cfg(feature = "long_term_memory")]
pub mod feature;

#[cfg(feature = "long_term_memory")]
use feature::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LongTermMemory {
    #[default]
    None,
    #[cfg(feature = "long_term_memory")]
    Some(MemoryThread),
}

#[cfg(feature = "long_term_memory")]
impl From<MemoryThread> for LongTermMemory {
    fn from(thread: MemoryThread) -> Self {
        LongTermMemory::Some(thread)
    }
}
