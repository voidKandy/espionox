use super::super::init::DbPool;
use super::super::models::file::*;
use sqlx::postgres::PgQueryResult;

pub async fn get_file(pool: &DbPool, params: GetFileParams) -> anyhow::Result<FileModelSql> {
    let query = sqlx::query_as::<_, FileModelSql>("SELECT * FROM files WHERE filepath = $1");

    match query.bind(params.filepath).fetch_one(pool.as_ref()).await {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn get_files_by_threadname(
    pool: &DbPool,
    threadname: &str,
) -> anyhow::Result<Vec<FileModelSql>, sqlx::Error> {
    let query = format!("SELECT * FROM files WHERE thread_name = {}", threadname);

    let result = sqlx::query_as::<_, FileModelSql>(&query)
        .fetch_all(pool.as_ref())
        .await;

    match result {
        Ok(files) => Ok(files),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_file(pool: &DbPool, file: CreateFileBody) -> anyhow::Result<PgQueryResult> {
    let query = "INSERT INTO files (id, thread_name, filepath, parent_dir_path, summary, summary_embedding) VALUES ($1, $2, $3, $4, $5, $6)";
    match sqlx::query(query)
        .bind(file.id)
        .bind(file.thread_name)
        .bind(file.filepath)
        .bind(file.parent_dir_path)
        .bind(file.summary)
        .bind(file.summary_embedding)
        .execute(pool.as_ref())
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_file(pool: &DbPool, params: DeleteFileParams) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM files WHERE filepath = $1");
    match sqlx::query(&query)
        .bind(params.filepath)
        .execute(pool.as_ref())
        .await
    {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
