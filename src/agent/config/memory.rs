use crate::database::models::file::CreateFileBody;
use crate::database::models::file_chunks::{CreateFileChunkBody, FileChunkModelSql};
use crate::database::models::messages::{CreateMessageBody, GetMessageParams};
use crate::database::{
    handlers::{self, messages},
    init::DbPool,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::{self, info};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Memory {
    Remember(LoadedMemory),
    Forget,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum LoadedMemory {
    LongTerm(String),
    Cache,
}

impl LoadedMemory {
    thread_local! {
        static CACHED_MEMORY: RefCell<Vec<Value>> = RefCell::new(Vec::new());
        static DATA_POOL: Arc<DbPool> = Arc::new(DbPool::init_long_term());
    }

    #[tracing::instrument]
    pub fn get(&self) -> Vec<Value> {
        match self {
            LoadedMemory::Cache => Self::CACHED_MEMORY.with(|mem| {
                let st_mem = mem.borrow().clone();
                info!("Messages loaded from Cache: {:?}", st_mem);
                st_mem
            }),
            LoadedMemory::LongTerm(threadname) => {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    let pool = &Self::DATA_POOL.with(|poo| Arc::clone(poo));
                    match handlers::threads::get_thread(pool, threadname).await {
                        Ok(_) => {}
                        Err(err) => {
                            if matches!(
                                err.downcast_ref::<sqlx::Error>(),
                                Some(sqlx::Error::RowNotFound)
                            ) {
                                info!("Thread doesn't exist, creating thread named: {threadname}");
                                assert!(handlers::threads::post_thread(pool, threadname)
                                    .await
                                    .is_ok());
                            } else {
                                panic!("Error getting thread {err:?}");
                            }
                        }
                    }
                    let messages = messages::get_messages(
                        &pool,
                        GetMessageParams {
                            thread_name: threadname.to_string(),
                        },
                    )
                    .await
                    .expect("Failed to get messages from context");
                    messages.into_iter().map(|m| m.coerce_to_value()).collect()
                })
            }
        }
    }

    pub fn store_messages(&self, messages: &Vec<Value>) {
        match self {
            LoadedMemory::Cache => {
                Self::CACHED_MEMORY.with(|st_mem| {
                    *st_mem.borrow_mut() = messages.to_owned();
                });
            }
            LoadedMemory::LongTerm(threadname) => {
                let rt = Runtime::new().unwrap();
                rt.block_on(async {
                    for m in messages.iter() {
                        messages::post_message(
                            &Self::DATA_POOL.with(|poo| Arc::clone(poo)),
                            CreateMessageBody {
                                thread_name: threadname.to_string(),
                                role: m.get("role").expect("No role").to_string(),
                                content: m.get("content").expect("No content").to_string(),
                            },
                        )
                        .await
                        .expect("Failed to create message body from Value");
                    }
                });
            }
        };
    }

    pub fn store_file_tup(&self, file_tup: (CreateFileBody, Vec<CreateFileChunkBody>)) {
        match self {
            LoadedMemory::Cache => {}
            LoadedMemory::LongTerm(threadname) => {
                let rt = Runtime::new().unwrap();
                let pool = &Self::DATA_POOL.with(|poo| Arc::clone(poo));
                rt.block_on(async {
                    handlers::file::post_file(pool, file_tup.0)
                        .await
                        .expect("Failed to create file body from Value");
                    for chunk in file_tup.1 {
                        handlers::file_chunks::post_file_chunk(pool, chunk)
                            .await
                            .expect("Failed to post chunks");
                    }
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
                LoadedMemory::Cache => LoadedMemory::Cache.store_messages(messages),
                LoadedMemory::LongTerm(_) => loaded.store_messages(messages),
            },
            _ => {}
        }
    }
}
