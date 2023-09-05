use rust_bert::pipelines::sentence_embeddings::Embedding;

use super::{
    models::{file::*, file_chunks::*},
    vector_embeddings::EmbeddingVector,
    DbPool,
};
use crate::{
    agent::spo_agents::SummarizerAgent,
    core::{File, FileChunk},
    language_models::embed,
};

#[derive(Clone)]
pub struct CreateFileChunksVector(Vec<CreateFileChunkBody>);

impl AsRef<Vec<CreateFileChunkBody>> for CreateFileChunksVector {
    fn as_ref(&self) -> &Vec<CreateFileChunkBody> {
        &self.0
    }
}

impl Into<crate::core::File> for FileModelSql {
    fn into(self) -> File {
        let mut file = File::from(self.filepath.as_str());
        file.summary = Some(self.summary);
        file
    }
}

impl Into<crate::core::FileChunk> for FileChunkModelSql {
    fn into(self) -> FileChunk {
        let parent_filepath: Box<std::path::Path> =
            std::fs::canonicalize(std::path::Path::new(&self.parent_filepath))
                .expect("Failed to get parent filepath")
                .into();
        FileChunk {
            parent_filepath,
            content: self.content,
            index: self.idx,
        }
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
                file.summary = Some(sum.clone());
                tracing::info!("File summary got");
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
                parent_filepath: chunk.parent_filepath.display().to_string(),
                idx: chunk.index,
                content: chunk.content.clone(),
                content_embedding,
            });
        }
        Ok(CreateFileChunksVector(resulting_chunk_models))
    }
}

#[tracing::instrument(name = "Get similar file chunks from vector embedding" skip(pool, vector))]
pub async fn vector_query_file_chunks(
    pool: &DbPool,
    vector: Embedding,
    distance: u8,
) -> anyhow::Result<Vec<FileChunkModelSql>> {
    let vector: EmbeddingVector = vector.into();
    match sqlx::query_as::<_, FileChunkModelSql>(&format!(
        "SELECT * FROM file_chunks WHERE content_embedding <-> $1 < {};",
        distance
    ))
    .bind(&vector)
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(chunks) => {
            chunks
                .iter()
                .for_each(|chunk| tracing::info!("Chunk got: {:?} : {}", chunk.idx, chunk.content));
            Ok(chunks)
        }
        Err(err) => Err(err.into()),
    }
}

#[tracing::instrument(name = "Get similar files from vector embedding" skip(pool, vector))]
pub async fn vector_query_files(
    pool: &DbPool,
    vector: Embedding,
    distance: u8,
) -> anyhow::Result<Vec<FileModelSql>> {
    let vector: EmbeddingVector = vector.into();
    match sqlx::query_as::<_, FileModelSql>(&format!(
        "SELECT * FROM files WHERE summary_embedding <-> $1 < {};",
        distance
    ))
    .bind(&vector)
    .fetch_all(pool.as_ref())
    .await
    {
        Ok(files) => {
            files.iter().for_each(|file| {
                tracing::info!("File got: {:?} : {}", file.filepath, file.summary)
            });
            Ok(files)
        }
        Err(err) => Err(err.into()),
    }
}
