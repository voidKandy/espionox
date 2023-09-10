use super::super::MessageVector;
use crate::{
    context::Context,
    core::{File, FileChunk},
    database::{self, handlers, models, vector_embeddings::EmbeddingVector, DbPool},
    language_models::embed,
};
use std::{any::Any, thread};
use tokio::runtime::Runtime;

#[derive(PartialEq)]
pub enum EmbeddedType {
    File,
    Chunk,
}

pub struct EmbeddedCoreStruct {
    core_type: EmbeddedType,
    core_struct: Box<dyn Embedded>,
}

impl EmbeddedCoreStruct {
    pub fn get_type(&self) -> &EmbeddedType {
        &self.core_type
    }

    pub fn try_as_file(&self) -> anyhow::Result<File> {
        if self.core_type == EmbeddedType::Chunk {
            return Err(anyhow::anyhow!("Struct is of chunk type"));
        }

        if let Some(file) = self.core_struct.as_any().downcast_ref::<File>() {
            Ok(file.to_owned())
        } else {
            Err(anyhow::anyhow!("Failed to downcast to File"))
        }
    }

    pub fn try_as_chunk(&self) -> anyhow::Result<FileChunk> {
        if self.core_type == EmbeddedType::File {
            return Err(anyhow::anyhow!("Struct is of File type"));
        }

        if let Some(chunk) = self.core_struct.as_any().downcast_ref::<FileChunk>() {
            Ok(chunk.to_owned())
        } else {
            Err(anyhow::anyhow!("Failed to downcast to Chunk"))
        }
    }
}

pub trait Embedded: std::fmt::Debug {
    fn get_from_embedding(query: EmbeddingVector, pool: &DbPool) -> Vec<EmbeddedCoreStruct>
    where
        Self: Sized;
    fn as_any(&self) -> &dyn Any;
}

impl Embedded for File {
    fn get_from_embedding(query: EmbeddingVector, pool: &DbPool) -> Vec<EmbeddedCoreStruct> {
        let pool = pool.to_owned();
        let files = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let files: anyhow::Result<Vec<File>> =
                    match database::api::vector_query_files(&pool, query.into(), 5).await {
                        Ok(files_sql) => Ok(files_sql.into_iter().map(|sql| sql.into()).collect()),
                        Err(err) => Err(err.into()),
                    };
                files.unwrap()
            })
        })
        .join()
        .expect("Failed to join thread");
        let mut return_vec = vec![];
        files.into_iter().for_each(|f| {
            return_vec.push(EmbeddedCoreStruct {
                core_type: EmbeddedType::File,
                core_struct: Box::new(f),
            })
        });
        return_vec
    }

    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }
}

impl Embedded for FileChunk {
    fn get_from_embedding(query: EmbeddingVector, pool: &DbPool) -> Vec<EmbeddedCoreStruct> {
        let pool = pool.to_owned();
        let chunks = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let files: anyhow::Result<Vec<FileChunk>> =
                    match database::api::vector_query_file_chunks(&pool, query.into(), 5).await {
                        Ok(chunks_sql) => {
                            Ok(chunks_sql.into_iter().map(|sql| sql.into()).collect())
                        }
                        Err(err) => Err(err.into()),
                    };
                files.unwrap()
            })
        })
        .join()
        .expect("Failed to join thread");
        let mut return_vec = vec![];
        chunks.into_iter().for_each(|c| {
            return_vec.push(EmbeddedCoreStruct {
                core_type: EmbeddedType::Chunk,
                core_struct: Box::new(c),
            })
        });
        return_vec
    }

    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }
}

#[tracing::instrument(name = "Query files by summary embeddings")]
pub fn get_files_by_summary_embeddings(query: &str, pool: &DbPool) -> Vec<File> {
    let pool = pool.to_owned();
    let query_vector = embed(query).expect("Failed to embed query");
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let files: anyhow::Result<Vec<crate::core::File>> =
                match database::api::vector_query_files(&pool, query_vector, 5).await {
                    Ok(files_sql) => Ok(files_sql.into_iter().map(|sql| sql.into()).collect()),
                    Err(err) => Err(err.into()),
                };
            files.unwrap()
        })
    })
    .join()
    .expect("Failed to join thread")
}

pub fn get_active_long_term_threads(context: &Context) -> Result<Vec<String>, String> {
    let context = context.to_owned();
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            match handlers::threads::get_all_threads(context.pool()).await {
                Ok(threads) => Ok(threads),
                Err(err) => Err(format!("Couldn't get long term threads: {err:?}")),
            }
        })
    })
    .join()
    .expect("Failed to get long term threads")
}

#[tracing::instrument(name = "Save messages to database from threadname")]
pub fn save_messages_to_database(threadname: &str, messages: MessageVector, pool: &DbPool) {
    let messages = messages.to_owned();
    let threadname = threadname.to_owned();
    let pool = pool.to_owned();
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
pub fn get_messages_from_database(threadname: &str, pool: &DbPool) -> MessageVector {
    let pool = pool.to_owned();
    let threadname = threadname.to_owned();
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            match handlers::threads::get_thread(&pool, &threadname).await {
                Ok(_) => {}
                Err(err) => {
                    if matches!(
                        err.downcast_ref::<sqlx::Error>(),
                        Some(sqlx::Error::RowNotFound)
                    ) {
                        tracing::info!("Thread doesn't exist, creating thread named: {threadname}");
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
