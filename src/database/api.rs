use super::models::{file::*, file_chunks::*};
use crate::core::File;
pub fn sql_from_file(file: File, thread_name: &str) -> (CreateFileBody, Vec<CreateFileChunkBody>) {
    let parent_dir_path: String = file.filepath.parent().unwrap().display().to_string();
    let thread_name = thread_name.to_string();
    let file_id = uuid::Uuid::new_v4().to_string();

    let file_sql = CreateFileBody {
        id: file_id.clone(),
        thread_name,
        filepath: file.filepath.display().to_string(),
        parent_dir_path,
        summary: file.summary.to_owned(),
        summary_embedding: pgvector::Vector::from(file.summary_embedding.to_owned()),
    };
    let chunks_sql: Vec<CreateFileChunkBody> = file
        .chunks
        .to_owned()
        .into_iter()
        .map(|ch| CreateFileChunkBody {
            parent_file_id: file_id.to_owned(),
            idx: ch.index,
            content: ch.content,
            content_embedding: pgvector::Vector::from(ch.content_embedding),
        })
        .collect();
    (file_sql, chunks_sql)
}
