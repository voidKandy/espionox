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
    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer
            .as_mut_ref()
            .push(Message::new_standard(role, content));
    }

    pub fn buffer_as_string(&self) -> String {
        let mut output = String::new();
        self.buffer.as_ref().into_iter().for_each(|mess| {
            output.push_str(&format!("{}\n", mess));
        });
        format!("{}", output)
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
