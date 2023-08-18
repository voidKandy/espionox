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
    LongTerm(String),
    ShortTerm,
    Forget,
}

#[allow(unreachable_code)]
pub fn store_file_tup(file_tup: (CreateFileBody, Vec<CreateFileChunkBody>)) {
    // let rt = Runtime::new().unwrap();
    // let pool = &Self::DATA_POOL.with(|poo| Arc::clone(poo));
    // rt.block_on(async {
    //     handlers::file::post_file(pool, file_tup.0)
    //         .await
    //         .expect("Failed to create file body from Value");
    //     for chunk in file_tup.1 {
    //         handlers::file_chunks::post_file_chunk(pool, chunk)
    //             .await
    //             .expect("Failed to post chunks");
    //     }
    // });
}

impl Memory {
    thread_local! {
        static CACHED_MEMORY: RefCell<MessageVector> = RefCell::new(MessageVector::new(Vec::new()));
        static DATA_POOL: Arc<DbPool> = Arc::new(DbPool::init_long_term());
    }

    fn save_messages_to_database(threadname: &str, messages: MessageVector) {
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

    fn save_messages_to_cache(messages: MessageVector) {
        Self::CACHED_MEMORY.with(|st_mem| {
            info!("Cache before: {:?}", st_mem.borrow());
            st_mem
                .borrow_mut()
                .as_mut_ref()
                .append(messages.to_owned().as_mut_ref());
            info!("Cache pushed: {:?}", st_mem.borrow());
        });
    }

    #[tracing::instrument]
    fn get_messages_from_database(threadname: &str) -> MessageVector {
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
                            info!("Thread doesn't exist, creating thread named: {threadname}");
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

    fn get_messages_from_cache() -> MessageVector {
        Self::CACHED_MEMORY.with(|mem| {
            let st_mem = mem.borrow();
            info!("Messages loaded from Cache: {:?}", st_mem);
            st_mem.to_owned()
        })
    }

    /// THIS DOESNT REQUIRE SELF
    pub fn get_active_long_term_threads(&self) -> Result<Vec<String>, String> {
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match handlers::threads::get_all_threads(
                    &Self::DATA_POOL.with(|poo| Arc::clone(poo)),
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
            Memory::LongTerm(threadname) => Self::get_messages_from_database(threadname),
            Memory::ShortTerm => Self::get_messages_from_cache(),
            Memory::Forget => MessageVector::new(vec![]),
        }
    }
    pub fn save(&self, messages: MessageVector) {
        match self {
            Memory::LongTerm(threadname) => {
                Self::save_messages_to_database(threadname, messages);
            }
            Memory::ShortTerm => {
                Self::save_messages_to_cache(messages);
            }
            Memory::Forget => {}
        }
    }
}
