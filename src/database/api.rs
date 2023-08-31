use rust_bert::pipelines::sentence_embeddings::Embedding;

use super::{
    init::DatabaseSettings,
    models::{file::*, file_chunks::*},
    DbPool,
};
use crate::{
    core::{File, FileChunk},
    handler::integrations::SummarizerAgent,
    language_models::embed,
};

#[derive(Clone)]
pub struct CreateFileChunksVector(Vec<CreateFileChunkBody>);

impl AsRef<Vec<CreateFileChunkBody>> for CreateFileChunksVector {
    fn as_ref(&self) -> &Vec<CreateFileChunkBody> {
        &self.0
    }
}

impl CreateFileBody {
    #[tracing::instrument(name = "Build CreateFileBody Sql struct from File struct")]
    pub fn build_from(
        file: &mut File,
        thread_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let summary = match &file.summary {
            None => {
                tracing::info!("File has no summary, getting summary");
                let sum = SummarizerAgent::init().summarize(file);
                println!("{}", &sum);
                file.summary = Some(sum.clone());
                sum
            }
            Some(summary) => summary.to_string(),
        };
        let parent_dir_path: String = file.filepath.parent().unwrap().display().to_string();
        let summary_embedding =
            pgvector::Vector::from(embed(&summary).expect("Failed to create summary embedding"));
        let thread_name = thread_name.to_string();
        let id = uuid::Uuid::new_v4().to_string();
        Ok(CreateFileBody {
            id,
            thread_name,
            filepath: file.filepath.display().to_string(),
            parent_dir_path,
            summary,
            summary_embedding,
        })
    }
}

impl CreateFileChunksVector {
    #[tracing::instrument(
        name = "Build CreateFileChunksVector struct from Vector of FileChunk structs"
    )]
    pub fn build_from(
        file_chunks: Vec<FileChunk>,
        parent_file_id: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut resulting_chunk_models = vec![];
        let parent_file_id = parent_file_id.to_string();
        for chunk in file_chunks.iter() {
            let content_embedding = pgvector::Vector::from(
                embed(&chunk.content).expect("Failed to create summary embedding"),
            );
            resulting_chunk_models.push(CreateFileChunkBody {
                parent_file_id: parent_file_id.clone(),
                idx: chunk.index,
                content: chunk.content.clone(),
                content_embedding,
            });
        }
        Ok(CreateFileChunksVector(resulting_chunk_models))
    }
}

pub async fn query_vector_embeddings(pool: &DbPool, vector: Embedding) {
    let query = format!(
        "SELECT * FROM file_chunks WHERE content_embedding <-> '{:?}' < 5;",
        vector
    );
    let result = sqlx::query(&query).execute(pool.as_ref()).await;

    match result {
        Ok(chunks) => println!("Chunks got: {:?}", chunks),
        Err(err) => panic!("{}", err),
    }
}

#[tracing::instrument(name = "Check that given database exists" skip(pool, db_name))]
pub async fn check_db_exists(pool: &DbPool, db_name: &str) -> bool {
    let query = format!(
        "SELECT datname FROM pg_catalog.pg_database WHERE datname = '{}';",
        db_name
    );
    let result = sqlx::query(&query).fetch_optional(pool.as_ref()).await;

    match result {
        Ok(Some(_)) => {
            tracing::info!("Database does exist!");
            true
        }
        _ => {
            tracing::info!("Database does not exist!");
            false
        }
    }
}

#[tracing::instrument(name = "Initialize and migrate new database" skip(pool, settings))]
pub async fn init_and_migrate_db(pool: &DbPool, settings: DatabaseSettings) -> anyhow::Result<()> {
    sqlx::query(&format!("CREATE DATABASE {}", settings.database_name))
        .execute(pool.as_ref())
        .await?;
    sqlx::migrate!("./migrations")
        .run(pool.as_ref())
        .await
        .expect("Failed to migrate database.");
    Ok(())
}
