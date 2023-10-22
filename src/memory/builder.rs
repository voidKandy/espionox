use crate::configuration::ConfigEnv;

use super::*;

#[cfg(feature = "long_term_memory")]
use crate::features::long_term_memory::*;

pub struct MemoryBuilder {
    init_prompt: Option<MessageVector>,
    env: Option<ConfigEnv>,
    // env is mostly for testing, can't think of a reason a dev would want to
    // change the environment other than for that
    recall_mode: Option<RecallMode>,
    caching_mechanism: Option<CachingMechanism>,
    long_term_thread: Option<String>,
}

impl MemoryBuilder {
    pub fn new() -> Self {
        Self {
            init_prompt: None,
            env: None,
            recall_mode: None,
            caching_mechanism: None,
            long_term_thread: None,
        }
    }

    #[cfg(feature = "long_term_memory")]
    pub fn env(mut self, env: ConfigEnv) -> Self {
        self.env = Some(env);
        self
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
        self.long_term_thread = Some(threadname.to_string());
        self
    }

    pub fn finished(self) -> Memory {
        let cache = self.init_prompt.unwrap_or(MessageVector::init());
        #[cfg(feature = "long_term_memory")]
        if let Some(_threadname) = self.long_term_thread {
            tracing::info!(
                "Initializing long term memory with threadname: {}",
                _threadname
            );
            let pool = match self.env {
                Some(env) => DbPool::sync_init_pool(env),
                None => DbPool::default(),
            };
            let long_term = LongTermMemory::from(LtmHandler::init(pool, &_threadname));

            return Memory {
                cache,
                recall_mode: self.recall_mode.unwrap_or_default(),
                caching_mechanism: self.caching_mechanism.unwrap_or_default(),
                long_term,
            };
        }
        // Leave this elseless!!
        let long_term = LongTermMemory::None;
        return Memory {
            cache,
            recall_mode: self.recall_mode.unwrap_or_default(),
            caching_mechanism: self.caching_mechanism.unwrap_or_default(),
            long_term,
        };
    }
}
