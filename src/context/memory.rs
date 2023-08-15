use super::messages::*;
use crate::database::{
    handlers::{self, messages},
    init::DbPool,
    models::{
        file::CreateFileBody,
        file_chunks::CreateFileChunkBody,
        messages::{CreateMessageBody, GetMessageParams},
    },
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, sync::Arc, thread};
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
        static CACHED_MEMORY: RefCell<MessageVector> = RefCell::new(MessageVector::new(Vec::new()));
        static DATA_POOL: Arc<DbPool> = Arc::new(DbPool::init_long_term());
    }

    #[tracing::instrument]
    pub fn get_messages(&self) -> MessageVector {
        match self {
            LoadedMemory::Cache => Self::CACHED_MEMORY.with(|mem| {
                let st_mem = mem.borrow();
                info!("Messages loaded from Cache: {:?}", st_mem);
                st_mem.to_owned()
            }),

            LoadedMemory::LongTerm(threadname) => {
                let threadname = threadname.to_owned();
                thread::spawn(move || {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        let pool = Self::DATA_POOL.with(|poo| Arc::clone(poo));
                        match handlers::threads::get_thread(&pool, &threadname).await {
                            Ok(_) => {}
                            Err(err) => {
                                if matches!(
                                    err.downcast_ref::<sqlx::Error>(),
                                    Some(sqlx::Error::RowNotFound)
                                ) {
                                    info!(
                                        "Thread doesn't exist, creating thread named: {threadname}"
                                    );
                                    assert!(handlers::threads::post_thread(&pool, &threadname)
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
                        MessageVector::new(messages.into_iter().map(|m| m.into()).collect())
                    })
                })
                .join()
                .expect("Failed to get long term memory messages")
            }
        }
    }

    pub fn store_messages(&self, messages: &MessageVector) {
        match self {
            LoadedMemory::Cache => {
                Self::CACHED_MEMORY.with(|st_mem| {
                    info!("Cache before: {:?}", st_mem.borrow());
                    st_mem
                        .borrow_mut()
                        .as_mut_ref()
                        .append(messages.to_owned().as_mut_ref());
                    info!("Cache pushed: {:?}", st_mem.borrow());
                });
            }
            LoadedMemory::LongTerm(threadname) => {
                let messages = messages.to_owned();
                let threadname = threadname.to_owned();
                thread::spawn(move || {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async {
                        for m in messages.as_ref().iter() {
                            messages::post_message(
                                &Self::DATA_POOL.with(|poo| Arc::clone(poo)),
                                CreateMessageBody {
                                    thread_name: threadname.to_string(),
                                    role: m.role().to_owned(),
                                    content: m.content().to_owned(),
                                },
                            )
                            .await
                            .expect("Failed to store messages to long term memory");
                        }
                    });
                });
            }
        };
    }

    pub fn store_file_tup(&self, file_tup: (CreateFileBody, Vec<CreateFileChunkBody>)) {
        match self {
            LoadedMemory::Cache => {}
            LoadedMemory::LongTerm(_) => {
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
    pub fn get_active_long_term_threads(&self) -> Result<Vec<String>, String> {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match handlers::threads::get_all_threads(
                    &LoadedMemory::DATA_POOL.with(|poo| Arc::clone(poo)),
                )
                .await
                {
                    Ok(threads) => Ok(threads),
                    Err(err) => Err(format!("Couldn't get long term threads: {err:?}")),
                }
            })
        })
        .join()
        .expect("Failed to get long term threads")
    }

    pub fn load(&self) -> MessageVector {
        match self {
            Memory::Remember(memory) => memory.get_messages(),
            Memory::Forget => MessageVector::new(vec![]),
        }
    }
    pub fn save(&self, messages: MessageVector) {
        match self {
            Memory::Remember(memory) => {
                memory.store_messages(&messages);
            }
            Memory::Forget => {}
        }
    }
}
