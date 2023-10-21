pub mod database;
pub use database::{
    api, handlers,
    init::DbPool,
    models::{self, file::CreateFileBody, messages::MessageModelSql},
};

use crate::{
    core::{Directory, File},
    language_models::embed,
    memory::{
        messages::{Message, MessageVector},
        FlatType, FlattenStruct, FlattenedCachedStruct,
    },
};
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug)]
pub struct LtmHandler {
    pool: Arc<DbPool>,
    current_thread: String,
}

impl PartialEq for LtmHandler {
    fn eq(&self, other: &Self) -> bool {
        self.current_thread == other.current_thread
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

impl LtmHandler {
    pub fn init(pool: DbPool, current_thread: &str) -> Self {
        let pool = Arc::new(pool);
        let current_thread = current_thread.to_string();
        Self {
            pool,
            current_thread,
        }
    }

    pub fn current_thread(&self) -> &String {
        &self.current_thread
    }

    pub fn switch_thread(&mut self, name: &str) {
        self.current_thread = name.to_string();
    }

    pub fn pool(&self) -> DbPool {
        self.pool.deref().to_owned()
    }

    pub async fn get_active_long_term_threads(&self) -> Result<Vec<String>, String> {
        let pool = self.pool.to_owned();
        match handlers::threads::get_all_threads(&pool).await {
            Ok(threads) => Ok(threads),
            Err(err) => Err(format!("Couldn't get long term threads: {err:?}")),
        }
    }

    #[tracing::instrument(name = "Save cached structs to database")]
    pub async fn save_cached_structs(&self, structs: Vec<FlattenedCachedStruct>) {
        let structs = structs.to_owned();
        let current_thread = self.current_thread.to_owned();
        let pool = self.pool.to_owned();
        for flat in structs.into_iter() {
            match flat.flat_type {
                FlatType::File => {
                    let mut file = File::rebuild(flat).unwrap();
                    let sql = CreateFileBody::build_from(&mut file, &current_thread)
                        .expect("Failed to create sql body");
                    handlers::file::post_file(&pool, sql)
                        .await
                        .expect("Failed to post single file");
                }
                FlatType::Directory => {
                    let dir = Directory::rebuild(flat).unwrap();
                    for mut file in dir.files {
                        let sql = CreateFileBody::build_from(&mut file, &current_thread)
                            .expect("Failed to create sql body");
                        handlers::file::post_file(&pool, sql)
                            .await
                            .expect("Failed to post file in directory");
                    }
                }
            }
        }
    }

    #[tracing::instrument(name = "Save messages to database from current_thread")]
    pub async fn save_messages_to_database(&self, messages: MessageVector) {
        let messages = messages.to_owned();
        let current_thread = self.current_thread.to_owned();
        let pool = self.pool.to_owned();
        for m in messages.as_ref().iter() {
            handlers::messages::post_message(
                &pool,
                models::messages::CreateMessageBody {
                    thread_name: current_thread.to_string(),
                    role: m.role().to_string(),
                    content: m.content().unwrap().to_owned(),
                },
            )
            .await
            .expect("Failed to store messages to long term memory");
        }
    }

    #[tracing::instrument(name = "Get messages from database from current_thread")]
    pub async fn get_messages_from_database(&self) -> MessageVector {
        let pool = self.pool.to_owned();
        let current_thread = self.current_thread.to_owned();
        match handlers::threads::get_thread(&pool, &current_thread.to_owned()).await {
            Ok(_) => {}
            Err(err) => {
                if matches!(
                    err.downcast_ref::<sqlx::Error>(),
                    Some(sqlx::Error::RowNotFound)
                ) {
                    tracing::info!(
                        "Thread doesn't exist, creating thread named: {current_thread:?}"
                    );
                    assert!(handlers::threads::post_thread(&pool, &current_thread)
                        .await
                        .is_ok());
                } else {
                    panic!("Error getting thread {err:?}");
                }
            }
        }
        let messages = handlers::messages::get_messages_by_threadname(&pool, &current_thread)
            .await
            .expect("Failed to get messages from context");
        MessageVector::from(
            messages
                .into_iter()
                .map(|m| m.into())
                .collect::<Vec<Message>>(),
        )
    }

    #[tracing::instrument(name = "Query files by summary embeddings")]
    pub async fn get_files_by_summary_embeddings(&self, query: &str) -> Vec<File> {
        let pool = self.pool.to_owned();
        let query_vector = embed(query).expect("Failed to embed query");
        let files: anyhow::Result<Vec<crate::core::File>> =
            match api::vector_query_files(&pool, query_vector, 5).await {
                Ok(files_sql) => Ok(files_sql.into_iter().map(|sql| sql.into()).collect()),
                Err(err) => Err(err.into()),
            };
        files.unwrap()
    }
}
