pub mod integrations;
pub mod memory;
pub mod messages;

pub use memory::*;
pub use messages::*;

use serde::{Deserialize, Serialize};

use crate::{configuration::ConfigEnv, database::DbPool};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Context {
    pub memory: Memory,
    pub buffer: MessageVector,
    #[serde(skip_serializing, skip_deserializing)]
    pub env: ConfigEnv,
    #[serde(skip_serializing, skip_deserializing)]
    db_pool: DbPool,
}

impl Context {
    pub fn build(memory: Memory, env: ConfigEnv) -> Context {
        let db_pool = DbPool::sync_init_pool(env.to_owned());
        Context {
            buffer: memory.load(Some(&db_pool)),
            memory,
            env,
            db_pool,
        }
    }

    pub fn pool(&self) -> &DbPool {
        &self.db_pool
    }

    pub fn save_buffer(&self) {
        let buf_difference = MessageVector::new(
            self.buffer
                .as_ref()
                .iter()
                .filter(|&value| {
                    !self
                        .memory
                        .load(Some(&self.db_pool))
                        .as_ref()
                        .contains(value)
                })
                .cloned()
                .collect(),
        );
        self.memory.save(buf_difference, Some(&self.db_pool));
    }
}
