use super::super::{
    long_term::database::{self, handlers, models, vector_embeddings::EmbeddingVector, DbPool},
    MessageVector,
};
use crate::{
    core::{File, FileChunk},
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
