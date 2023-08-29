use super::{
    init::DatabaseSettings,
    models::{file::*, file_chunks::*},
    DbPool,
};
use crate::{
    core::{File, FileChunk},
    handler::integrations::{BufferDisplay, SummarizerAgent},
    language_models::embed,
};

#[derive(Clone)]
pub struct CreateFileChunksVector(Vec<CreateFileChunkBody>);

#[derive(Clone)]
pub struct CreateFileAndChunksSql {
    pub file: CreateFileBody,
    chunks: CreateFileChunksVector,
}

impl AsRef<Vec<CreateFileChunkBody>> for CreateFileChunksVector {
    fn as_ref(&self) -> &Vec<CreateFileChunkBody> {
        &self.0
    }
}

impl CreateFileBody {
    pub fn build_from(
        file: &mut File,
        thread_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let summary = match &file.summary {
            None => {
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

impl CreateFileAndChunksSql {
    pub fn from_file_and_threadname(mut f: File, thread_name: &str) -> Self {
        let file_chunks = f.chunks.clone();
        let file = CreateFileBody::build_from(&mut f, thread_name)
            .expect("Failed to build create file sql body");
        let chunks = CreateFileChunksVector::build_from(file_chunks, &file.id)
            .expect("Failed to build create file chunks sql body");
        CreateFileAndChunksSql { file, chunks }
    }

    fn chunks(&self) -> Vec<CreateFileChunkBody> {
        self.chunks.as_ref().to_vec()
    }

    pub async fn insert_into_db(&self, pool: &DbPool) -> Result<(), sqlx::Error> {
        super::handlers::file::post_file(&pool, self.clone().file)
            .await
            .expect("Failed to post file");
        for chunk in self.chunks() {
            super::handlers::file_chunks::post_file_chunk(&pool, chunk)
                .await
                .expect("Failed to post chunk");
        }
        Ok(())
    }
}

pub async fn check_db_exists(pool: &DbPool, db_name: &str) -> bool {
    let query = format!(
        "SELECT datname FROM pg_catalog.pg_database WHERE datname = {};",
        db_name
    );
    let result = sqlx::query(&query).fetch_optional(pool.as_ref()).await;

    match result {
        Ok(Some(_)) => true,
        _ => false,
    }
}

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
