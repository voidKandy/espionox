use super::*;

#[cfg(feature = "long_term_memory")]
use super::long_term::feature::*;

pub struct MemoryBuilder {
    init_prompt: Option<MessageVector>,
    recall_mode: Option<RecallMode>,
    caching_mechanism: Option<CachingMechanism>,
    long_term_memory: Option<LongTermMemory>,
}

impl MemoryBuilder {
    pub fn new() -> Self {
        Self {
            init_prompt: None,
            recall_mode: None,
            caching_mechanism: None,
            long_term_memory: None,
        }
    }

    pub fn recall(mut self, recall: RecallMode) -> Self {
        self.recall_mode = Some(recall);
        self
    }

    pub fn caching_mechanism(mut self, caching_mech: CachingMechanism) -> Self {
        self.caching_mechanism = Some(caching_mech);
        self
    }

    pub fn init_prompt(mut self, init_prompt: MessageVector) -> Self {
        self.init_prompt = Some(init_prompt);
        self
    }

    #[cfg(feature = "long_term_memory")]
    pub fn long_term_thread(mut self, threadname: &str) -> Self {
        let pool = DbPool::default();
        self.long_term_memory = Some(LongTermMemory::from(MemoryThread::init(pool, threadname)));
        self
    }

    pub fn finished(self) -> Memory {
        Memory {
            cache: self.init_prompt.unwrap_or_else(MessageVector::init),
            recall_mode: self.recall_mode.unwrap_or_default(),
            caching_mechanism: self.caching_mechanism.unwrap_or_default(),
            long_term: self.long_term_memory.unwrap_or_default(),
        }
    }
}
