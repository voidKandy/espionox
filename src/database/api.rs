use super::{
    init::DatabaseSettings,
    models::{file::*, file_chunks::*},
    DbPool,
};
use crate::core::File;

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

// pub fn build_from_threadname(pool: &DbPool, threadname: &str) -> Context {
//     let files = get_files_by_threadname(pool, threadname);
//     let messages = get_messages_by_threadname(pool, threadname);
// }
