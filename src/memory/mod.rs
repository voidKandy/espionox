mod builder;
pub mod cache;
pub mod long_term;
pub mod messages;

#[cfg(feature = "long_term_memory")]
use crate::features::long_term_memory::DbPool;
use crate::{agents::spo_agents::SummarizerAgent, errors::error_chain_fmt};
use builder::MemoryBuilder;
pub use cache::*;
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
    cache: MemoryCache,
    long_term: LongTermMemory,
    recall_mode: RecallMode,
    caching_mechanism: CachingMechanism,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RecallMode {
    #[default]
    Default,
}

impl Memory {
    #[cfg(feature = "long_term_memory")]
    #[tracing::instrument(name = "Save cache to database")]
    pub async fn save_cache_to_long_term(&self) -> Result<(), anyhow::Error> {
        if let LongTermMemory::Some(ltm) = &self.long_term {
            let messages = self.cache.messages.clone();
            tracing::info!("{} messages to be saved", messages.len());
            ltm.save_messages_to_database(messages).await;
            if let Some(flattened_structs) = &self.cache.cached_structs {
                tracing::info!("{} structs to be saved", flattened_structs.len());
                ltm.save_cached_structs(flattened_structs.to_vec()).await;
            }
            return Ok(());
        }
        Err(anyhow::anyhow!("No long term memory in memory"))
    }

    pub fn build() -> MemoryBuilder {
        MemoryBuilder::new()
    }

    pub fn cache(&self) -> &MessageVector {
        &self.cache.messages
    }

    pub fn recall_mode(&self) -> &RecallMode {
        &self.recall_mode
    }

    pub fn caching_mechanism(&self) -> &CachingMechanism {
        &self.caching_mechanism
    }

    pub fn force_push_message_to_cache(&mut self, message: Message) {
        self.cache.messages.as_mut().push(message);
    }

    pub fn force_push_cached_structs_to_messages(&mut self) {
        if let Some(structs) = self.cache.cached_structs.to_owned() {
            for s in structs.iter() {
                self.force_push_message_to_cache(s.with_default_role())
            }
        }
    }

    pub fn push_flattened_struct_to_cache(&mut self, obj: FlattenedStruct) {
        match &mut self.cache.cached_structs {
            Some(structs) => {
                structs.push(obj);
            }
            None => {
                self.cache.cached_structs = Some(vec![obj]);
            }
        }
    }

    pub fn flatten_struct_to_cache(&mut self, obj: impl FlattenStruct) {
        let flat = obj.flatten();
        self.push_flattened_struct_to_cache(flat);
    }

    pub async fn push_to_message_cache(&mut self, role: &str, displayable: impl ToMessage) {
        if self.cache_size_limit_reached() {
            self.handle_oversized_cache().await;
        }
        self.cache
            .messages
            .as_mut()
            .push(displayable.to_message(role.to_string().into()));
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
        self.cache.messages.len_excluding_system_prompt() >= self.caching_mechanism.limit()
    }

    #[async_recursion::async_recursion]
    async fn handle_oversized_cache(&mut self) {
        match self.caching_mechanism {
            CachingMechanism::Forgetful => self.cache.messages.reset_to_system_prompt(),
            CachingMechanism::SummarizeAtLimit { save_to_lt, .. } => {
                if save_to_lt {
                    self.save_cache_to_long_term()
                        .await
                        .expect("Failed to save cache to long term when handling oversized cache");
                }
                let summary = SummarizerAgent::summarize_memory(
                    self.cache.messages.clone_sans_system_prompt(),
                )
                .await
                .expect("Failed to get memory summary");
                self.cache.messages.reset_to_system_prompt();
                self.force_push_message_to_cache(summary.to_message(MessageRole::System))
            }
        }
    }
}
