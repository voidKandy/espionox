use super::{long_term::*, message::*, traits::*, MessageVector};
#[cfg(feature = "long_term_memory")]
use crate::features::long_term_memory::DbPool;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
/// Memory contains:
/// * Cached MessageVector
/// * LongTermMemory, which is either None, or connected to a DbPool
/// * Recall Mode, which defines how the agent recalls memories (Messages)
/// * CachingMechanism, which defines how memories (Messages) are stored
pub struct Memory {
    cache: MessageVector,
    long_term: LongTermMemory,
    // recall_mode: RecallMode,
    // caching_mechanism: CachingMechanism,
}

impl From<MessageVector> for Memory {
    fn from(cache: MessageVector) -> Self {
        Self {
            cache,
            long_term: LongTermMemory::None,
        }
    }
}

impl Memory {
    pub fn new(init_prompt: &str, long_term: LongTermMemory) -> Self {
        Self {
            cache: MessageVector::new(init_prompt),
            long_term,
        }
    }

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

    pub fn cache(&self) -> &MessageVector {
        &self.cache
    }
    pub fn force_push_message_to_cache(&mut self, message: Message) {
        self.cache.as_mut().push(message);
    }

    // pub async fn push_to_message_cache(&mut self, role: Option<&str>, displayable: impl ToMessage) {
    //     if self.cache_size_limit_reached() {
    //         self.handle_oversized_cache().await;
    //     }
    //     let message = match role {
    //         Some(role) => {
    //             let role = role.to_string().into();
    //             displayable.to_message_with_role(role)
    //         }
    //         None => displayable.to_message(),
    //     };
    //     self.cache.as_mut().push(message);
    // }

    pub fn append_to_message_cache(&mut self, displayable_vec: impl Into<MessageVector>) {
        // if self.cache_size_limit_reached() {
        //     self.handle_oversized_cache().await;
        // }
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

    // #[allow(unused)]
    // #[async_recursion::async_recursion]
    // async fn handle_oversized_cache(&mut self) {
    //     let to_messages_opt = self.cache.get_structs();
    //     match self.caching_mechanism {
    //         CachingMechanism::Forgetful => self.cache.reset_to_system_prompt(),
    //         CachingMechanism::SummarizeAtLimit { save_to_lt, .. } => {
    //             #[cfg(feature = "long_term_memory")]
    //             if save_to_lt {
    //                 self.save_cache_to_long_term()
    //                     .await
    //                     .expect("Failed to save cache to long term when handling oversized cache");
    //             }
    //             let summary =
    //                 SummarizerAgent::summarize_memory(self.cache.clone_sans_system_prompt())
    //                     .await
    //                     .expect("Failed to get memory summary");
    //             self.cache.reset_to_system_prompt();
    //             self.force_push_message_to_cache(
    //                 summary.to_message_with_role(SummarizerAgent::message_role()),
    //             )
    //         }
    //     }
    //     if let Some(to_messages) = to_messages_opt {
    //         for to_message in to_messages.into_iter() {
    //             self.cache.push(to_message.to_message());
    //         }
    //     }
    // }
}
