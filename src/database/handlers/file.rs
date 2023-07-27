use super::super::init::DbPool;
use super::super::models::file::*;
use sqlx::postgres::PgQueryResult;

pub async fn get_file(pool: &DbPool, params: GetFileParams) -> anyhow::Result<FileModelSql> {
    let query = sqlx::query_as::<_, FileModelSql>("SELECT * FROM files WHERE filepath = $1");

    match query.bind(params.filepath).fetch_one(&pool.0).await {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_file(pool: &DbPool, file: CreateFileBody) -> anyhow::Result<PgQueryResult> {
    let query = "INSERT INTO files (id, thread_id, filepath, parent_dir_path, summary, summary_embedding) VALUES ($1, $2, $3, $4, $5, $6)";
    match sqlx::query(query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(file.thread_id)
        .bind(file.filepath)
        .bind(file.parent_dir_path)
        .bind(file.summary)
        .bind(file.summary_embedding)
        .execute(&pool.0)
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
        .execute(&pool.0)
        .await
    {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
