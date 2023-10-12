pub mod database;
pub use database::{
    api, handlers,
    init::DbPool,
    models::{self, messages::MessageModelSql},
};
use pollster::FutureExt as _;

use crate::{
    context::memory::messages::{Message, MessageVector},
    core::File,
    language_models::embed,
};
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug)]
pub struct MemoryThread {
    pool: Arc<DbPool>,
    threadname: String,
}

impl PartialEq for MemoryThread {
    fn eq(&self, other: &Self) -> bool {
        self.threadname == other.threadname
    }
}

#[cfg(feature = "long_term_memory")]
impl From<MessageModelSql> for Message {
    fn from(sql_model: MessageModelSql) -> Self {
        Message::Standard {
            role: sql_model.role.into(),
            content: sql_model.content,
        }
    }
}

impl MemoryThread {
    pub fn init(pool: DbPool, threadname: &str) -> Self {
        let pool = Arc::new(pool);
        let threadname = threadname.to_string();
        Self { pool, threadname }
    }

    pub fn threadname(&self) -> &String {
        &self.threadname
    }

    pub fn switch_thread(&mut self, name: &str) {
        self.threadname = name.to_string();
    }

    pub fn pool(&self) -> DbPool {
        self.pool.deref().to_owned()
    }

    pub fn get_active_long_term_threads(&self) -> Result<Vec<String>, String> {
        let pool = self.pool.to_owned();
        let future = async {
            match handlers::threads::get_all_threads(&pool).await {
                Ok(threads) => Ok(threads),
                Err(err) => Err(format!("Couldn't get long term threads: {err:?}")),
            }
        };
        future.block_on()
    }

    #[tracing::instrument(name = "Save messages to database from threadname")]
    pub fn save_messages_to_database(&self, messages: MessageVector) {
        let messages = messages.to_owned();
        let threadname = self.threadname.to_owned();
        let pool = self.pool.to_owned();
        let future = async {
            for m in messages.as_ref().iter() {
                handlers::messages::post_message(
                    &pool,
                    models::messages::CreateMessageBody {
                        thread_name: threadname.to_string(),
                        role: m.role().to_string(),
                        content: m.content().unwrap().to_owned(),
                    },
                )
                .await
                .expect("Failed to store messages to long term memory");
            }
        };
        future.block_on();
    }

    #[tracing::instrument(name = "Get messages from database from threadname")]
    pub fn get_messages_from_database(&self) -> MessageVector {
        let pool = self.pool.to_owned();
        let threadname = self.threadname.to_owned();
        let future = async {
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
            MessageVector::from(
                messages
                    .into_iter()
                    .map(|m| m.into())
                    .collect::<Vec<Message>>(),
            )
        };
        future.block_on()
    }

    #[tracing::instrument(name = "Query files by summary embeddings")]
    pub fn get_files_by_summary_embeddings(&self, query: &str) -> Vec<File> {
        let pool = self.pool.to_owned();
        let query_vector = embed(query).expect("Failed to embed query");
        let future = async {
            let files: anyhow::Result<Vec<crate::core::File>> =
                match api::vector_query_files(&pool, query_vector, 5).await {
                    Ok(files_sql) => Ok(files_sql.into_iter().map(|sql| sql.into()).collect()),
                    Err(err) => Err(err.into()),
                };
            files.unwrap()
        };
        future.block_on()
    }
}
