mod builder;
pub mod caching;
pub mod long_term;
pub mod messages;

#[cfg(feature = "long_term_memory")]
use crate::features::long_term_memory::DbPool;
use crate::{agents::spo_agents::SummarizerAgent, errors::error_chain_fmt};
use builder::MemoryBuilder;
pub use caching::*;
use long_term::*;
pub use messages::*;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Memory {
    cache: MessageVector,
    long_term: LongTermMemory,
    recall_mode: RecallMode,
    caching_mechanism: CachingMechanism,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum RecallMode {
    #[default]
    Default,
}

impl Memory {
    #[cfg(feature = "long_term_memory")]
    #[tracing::instrument(name = "Save cache to database")]
    pub async fn save_cache_to_long_term(&self) -> Result<(), anyhow::Error> {
        if let LongTermMemory::Some(ltm) = &self.long_term {
            let messages = self.cache.clone();
            tracing::info!("{} messages to be saved", messages.len());
            ltm.save_messages_to_database(messages).await;
            if let Some(structs) = self.cache.get_structs() {
                tracing::info!("{} structs to be saved", structs.len());
                ltm.save_cached_structs(structs).await;
            }
            return Ok(());
        }
        Err(anyhow::anyhow!("No long term memory in memory"))
    }

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

    pub fn force_push_message_to_cache(&mut self, message: Message) {
        self.cache.as_mut().push(message);
    }

    pub async fn push_to_message_cache(&mut self, role: Option<&str>, displayable: impl ToMessage) {
        if self.cache_size_limit_reached() {
            self.handle_oversized_cache().await;
        }
        let message = match role {
            Some(role) => {
                let role = role.to_string().into();
                displayable.to_message_with_role(role)
            }
            None => displayable.to_message(),
        };
        self.cache.as_mut().push(message);
    }

    pub async fn append_to_message_cache(&mut self, displayable_vec: impl Into<MessageVector>) {
        if self.cache_size_limit_reached() {
            self.handle_oversized_cache().await;
        }
        self.cache.append(displayable_vec.into());
    }

    #[cfg(feature = "long_term_memory")]
    pub fn long_term_thread(&self) -> Option<String> {
        match &self.long_term {
            LongTermMemory::None => None,
            LongTermMemory::Some(mem) => Some(mem.current_thread().to_string()),
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
        self.cache.chat_count() >= self.caching_mechanism.limit()
    }

    #[allow(unused)]
    #[async_recursion::async_recursion]
    async fn handle_oversized_cache(&mut self) {
        let to_messages_opt = self.cache.get_structs();
        match self.caching_mechanism {
            CachingMechanism::Forgetful => self.cache.reset_to_system_prompt(),
            CachingMechanism::SummarizeAtLimit { save_to_lt, .. } => {
                #[cfg(feature = "long_term_memory")]
                if save_to_lt {
                    self.save_cache_to_long_term()
                        .await
                        .expect("Failed to save cache to long term when handling oversized cache");
                }
                let summary =
                    SummarizerAgent::summarize_memory(self.cache.clone_sans_system_prompt())
                        .await
                        .expect("Failed to get memory summary");
                self.cache.reset_to_system_prompt();
                self.force_push_message_to_cache(
                    summary.to_message_with_role(SummarizerAgent::message_role()),
                )
            }
        }
        if let Some(to_messages) = to_messages_opt {
            for to_message in to_messages.into_iter() {
                self.cache.push(to_message.to_message());
            }
        }
    }
}
