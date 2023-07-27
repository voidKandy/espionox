use crate::database::models;
use crate::database::{handlers::message, init::DbPool};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use tokio::runtime::Runtime;
use tracing::{self, info};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Memory {
    Remember(LoadedMemory),
    Forget,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LoadedMemory {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    LongTerm(DbPool),
    Cache,
}

impl LoadedMemory {
    thread_local! {
        static CACHED_MEMORY: RefCell<Vec<Value>> = RefCell::new(Vec::new());
    }

    #[tracing::instrument]
    pub fn get(&self) -> Vec<Value> {
        match self {
            LoadedMemory::Cache => Self::CACHED_MEMORY.with(|mem| {
                let st_mem = mem.borrow().clone();
                info!("Messages loaded from Cache: {:?}", st_mem);
                st_mem
            }),
            LoadedMemory::LongTerm(pool) => {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    match message::get_messages(
                        &pool,
                        models::message::GetMessageParams {
                            thread_id: "main".to_string(),
                        },
                    )
                    .await
                    {
                        Ok(messages) => messages
                            .into_iter()
                            .map(|m| {
                                let value = m.coerce_to_value();
                                info!("Message loaded from LongTerm: {:?}", value);
                                value
                            })
                            .collect(),
                        Err(err) => panic!("Failed to get messages for context: {err:?}"),
                    }
                })
            }
        }
    }

    fn store(&self, messages: &Vec<Value>) {
        match self {
            LoadedMemory::LongTerm(pool) => {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    for m in messages.iter() {
                        message::post_message(
                            &pool,
                            models::message::CreateMessageBody {
                                thread_id: "main".to_string(),
                                role: m.get("role").expect("No role").to_string(),
                                content: m.get("content").expect("No content").to_string(),
                            },
                        )
                        .await
                        .expect("Failed to create message body from Value");
                    }
                });
            }
            LoadedMemory::Cache => {
                Self::CACHED_MEMORY.with(|st_mem| {
                    *st_mem.borrow_mut() = messages.to_owned();
                });
            }
        };
    }
}

impl Memory {
    pub fn load(&self) -> Vec<Value> {
        match self {
            Memory::Remember(memory) => memory.get(),
            Memory::Forget => vec![],
        }
    }
    pub fn save(&self, messages: &Vec<Value>) {
        match self {
            Memory::Remember(loaded) => match loaded {
                LoadedMemory::Cache => LoadedMemory::Cache.store(messages),
                LoadedMemory::LongTerm(_) => loaded.store(messages),
            },
            _ => {}
        }
    }
}
