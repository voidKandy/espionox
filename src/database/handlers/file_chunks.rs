use super::super::init::DbPool;
use super::super::models::file_chunks::*;
use sqlx::postgres::PgQueryResult;

pub async fn get_file_chunks(
    pool: &DbPool,
    params: GetFileChunkParams,
) -> anyhow::Result<Vec<FileChunkModelSql>> {
    let query = sqlx::query_as::<_, FileChunkModelSql>(
        "SELECT * FROM file_chunks WHERE parent_file_id = $1",
    );

    match query
        .bind(params.parent_file_id)
        .fetch_all(pool.as_ref())
        .await
    {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_file_chunk(
    pool: &DbPool,
    chunk: CreateFileChunkBody,
) -> anyhow::Result<PgQueryResult> {
    let query = "INSERT INTO file_chunks (id, parent_file_id, idx, content, content_embedding) VALUES ($1, $2, $3, $4, $5)";
    match sqlx::query(query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(chunk.parent_file_id)
        .bind(chunk.idx)
        .bind(chunk.content)
        .bind(chunk.content_embedding)
        .execute(pool.as_ref())
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_file_chunk(
    pool: &DbPool,
    params: DeleteFileChunkParams,
) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM file_chunks WHERE parent_file_id = $1 AND idx = $2");
    match sqlx::query(&query)
        .bind(params.parent_file_id)
        .bind(params.idx)
        .execute(pool.as_ref())
        .await
    {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
