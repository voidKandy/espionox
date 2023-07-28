use super::super::init::DbPool;
use super::super::models::error::*;
use sqlx::postgres::PgQueryResult;

pub async fn get_errors(
    pool: &DbPool,
    params: GetErrorParams,
) -> anyhow::Result<Vec<ErrorModelSql>> {
    let query = sqlx::query_as::<_, ErrorModelSql>("SELECT * FROM errors WHERE thread_name = $1");

    match query.bind(params.thread_name).fetch_all(&pool.0).await {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_error(pool: &DbPool, chunk: CreateErrorBody) -> anyhow::Result<PgQueryResult> {
    let query =
        "INSERT INTO errors (id, thread_name, content, content_embedding) VALUES ($1, $2, $3, $4)";
    match sqlx::query(query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(chunk.thread_name)
        .bind(chunk.content)
        .bind(chunk.content_embedding)
        .execute(&pool.0)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_error(
    pool: &DbPool,
    params: DeleteErrorParams,
) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM errors WHERE id = $1");
    match sqlx::query(&query).bind(params.id).execute(&pool.0).await {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
