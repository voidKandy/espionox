pub mod database;

use database::{api, handlers, models, DbPool};

use super::{Memory, MessageVector};
use crate::{core::File, language_models::embed};
use std::{sync::Arc, thread};
use tokio::runtime::Runtime;

#[derive(Clone, Debug)]
pub struct LongTermMemory {
    pub pool: Arc<DbPool>,
    threadname: Option<String>,
}

impl Memory for LongTermMemory {
    fn save(&self, messages: MessageVector) {
        self.save_messages_to_database(messages);
    }

    fn load(&self) -> MessageVector {
        self.get_messages_from_database()
    }
}

impl LongTermMemory {
    pub fn init(pool: Arc<DbPool>) -> Self {
        Self {
            pool,
            threadname: None,
        }
    }

    pub fn switch_thread(&mut self, name: &str) {
        self.threadname = Some(name.to_string());
    }

    pub fn threadname(&self) -> Option<String> {
        self.threadname.to_owned()
    }

    pub fn get_active_long_term_threads(&self) -> Result<Vec<String>, String> {
        let pool = self.pool.to_owned();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match handlers::threads::get_all_threads(&pool).await {
                    Ok(threads) => Ok(threads),
                    Err(err) => Err(format!("Couldn't get long term threads: {err:?}")),
                }
            })
        })
        .join()
        .expect("Failed to get long term threads")
    }

    #[tracing::instrument(name = "Save messages to database from threadname")]
    pub fn save_messages_to_database(&self, messages: MessageVector) {
        let messages = messages.to_owned();
        let threadname = self.threadname.to_owned().expect("No threadname");
        let pool = self.pool.to_owned();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                for m in messages.as_ref().iter() {
                    handlers::messages::post_message(
                        &pool,
                        models::messages::CreateMessageBody {
                            thread_name: threadname.to_string(),
                            role: m.role().to_owned(),
                            content: m.content().unwrap().to_owned(),
                        },
                    )
                    .await
                    .expect("Failed to store messages to long term memory");
                }
            });
        });
    }

    #[tracing::instrument(name = "Get messages from database from threadname")]
    pub fn get_messages_from_database(&self) -> MessageVector {
        let pool = self.pool.to_owned();
        let threadname = self.threadname.to_owned().expect("No threadname");
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match handlers::threads::get_thread(&pool, &threadname.to_owned()).await {
                    Ok(_) => {}
                    Err(err) => {
                        if matches!(
                            err.downcast_ref::<sqlx::Error>(),
                            Some(sqlx::Error::RowNotFound)
                        ) {
                            tracing::info!(
                                "Thread doesn't exist, creating thread named: {threadname:?}"
                            );
                            assert!(handlers::threads::post_thread(&pool, &threadname)
                                .await
                                .is_ok());
                        } else {
                            panic!("Error getting thread {err:?}");
                        }
                    }
                }
                let messages = handlers::messages::get_messages_by_threadname(&pool, &threadname)
                    .await
                    .expect("Failed to get messages from context");
                MessageVector::new(messages.into_iter().map(|m| m.into()).collect())
            })
        })
        .join()
        .expect("Failed to get long term memory messages")
    }

    #[tracing::instrument(name = "Query files by summary embeddings")]
    pub fn get_files_by_summary_embeddings(&self, query: &str) -> Vec<File> {
        let pool = self.pool.to_owned();
        let query_vector = embed(query).expect("Failed to embed query");
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let files: anyhow::Result<Vec<crate::core::File>> =
                    match api::vector_query_files(&pool, query_vector, 5).await {
                        Ok(files_sql) => Ok(files_sql.into_iter().map(|sql| sql.into()).collect()),
                        Err(err) => Err(err.into()),
                    };
                files.unwrap()
            })
        })
        .join()
        .expect("Failed to join thread")
    }
}
