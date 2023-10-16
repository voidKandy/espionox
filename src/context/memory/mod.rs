mod builder;
pub mod long_term;
pub mod messages;

#[cfg(feature = "long_term_memory")]
use long_term::feature::DbPool;

use crate::{agent::spo_agents::SummarizerAgent, errors::error_chain_fmt};
use builder::MemoryBuilder;
use long_term::*;
pub use messages::*;

#[derive(thiserror::Error)]
pub enum MemoryError {
    BuildError(String),
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryVariant {
    ShortTerm,
    LongTerm,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Memory {
    cache: MessageVector,
    long_term: LongTermMemory,
    recall_mode: RecallMode,
    caching_mechanism: CachingMechanism,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CachingMechanism {
    Forgetful,
    SummarizeAtLimit { limit: usize, save_to_lt: bool },
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RecallMode {
    #[default]
    Default,
}

impl Default for CachingMechanism {
    fn default() -> Self {
        Self::default_summary_at_limit()
    }
}

impl CachingMechanism {
    pub fn limit(&self) -> usize {
        match self {
            Self::Forgetful => 2, // Only allows for user in agent out
            Self::SummarizeAtLimit { limit, .. } => *limit as usize,
        }
    }
    pub fn long_term_enabled(&self) -> bool {
        match self {
            Self::Forgetful => false,
            Self::SummarizeAtLimit { save_to_lt, .. } => *save_to_lt,
        }
    }
    pub fn default_summary_at_limit() -> Self {
        let limit = 50;
        let save_to_lt = false;
        CachingMechanism::SummarizeAtLimit { limit, save_to_lt }
    }
}
impl Memory {
    pub fn build() -> MemoryBuilder {
        MemoryBuilder::new()
    }

    pub fn cache(&self) -> &MessageVector {
        &self.cache
    }

    pub fn recall_mode(&self) -> &RecallMode {
        &self.recall_mode
    }

    pub fn caching_mechanism(&self) -> &CachingMechanism {
        &self.caching_mechanism
    }

    pub fn force_push_message_to_cache(&mut self, role: &str, displayable: impl ToMessage) {
        self.cache
            .as_mut()
            .push(displayable.to_message(role.to_string().into()));
    }

    pub async fn push_to_message_cache(&mut self, role: &str, displayable: impl ToMessage) {
        if self.cache_size_limit_reached() {
            self.handle_oversized_cache().await;
        }
        self.cache
            .as_mut()
            .push(displayable.to_message(role.to_string().into()));
    }

    #[cfg(feature = "long_term_memory")]
    pub fn long_term_thread(&self) -> Option<String> {
        match &self.long_term {
            LongTermMemory::None => None,
            LongTermMemory::Some(mem) => Some(mem.threadname().to_string()),
        }
    }

    #[cfg(feature = "long_term_memory")]
    pub fn long_term_pool(&self) -> Option<DbPool> {
        match &self.long_term {
            LongTermMemory::None => None,
            LongTermMemory::Some(mem) => Some(mem.pool()),
        }
    }

    fn cache_size_limit_reached(&self) -> bool {
        self.cache.len_excluding_system_prompt() >= self.caching_mechanism.limit()
    }

    #[async_recursion::async_recursion]
    async fn handle_oversized_cache(&mut self) {
        match self.caching_mechanism {
            CachingMechanism::Forgetful => self.cache.reset_to_system_prompt(),
            CachingMechanism::SummarizeAtLimit { save_to_lt, .. } => {
                let summary =
                    SummarizerAgent::summarize_memory(self.cache.clone_sans_system_prompt())
                        .await
                        .expect("Failed to get memory summary");
                self.cache.reset_to_system_prompt();
                self.cache.push(summary.to_message(MessageRole::System))
            }
        }
    }
}
