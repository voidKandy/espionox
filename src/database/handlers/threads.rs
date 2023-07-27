use super::super::init::DbPool;
use super::super::models::threads::*;
use sqlx::postgres::PgQueryResult;

pub async fn get_thread(pool: &DbPool, params: ThreadParams) -> anyhow::Result<ThreadModelSql> {
    match sqlx::query_as!(
        ThreadModelSql,
        "SELECT * FROM threads WHERE name = $1",
        params.name
    )
    .fetch_one(&pool.0)
    .await
    {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_thread(pool: &DbPool, params: ThreadParams) -> anyhow::Result<PgQueryResult> {
    let query = format!("INSERT INTO threads (id, name) VALUES ($1, $2)");
    match sqlx::query(&query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(params.name)
        .execute(&pool.0)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_thread(pool: &DbPool, params: ThreadParams) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM threads WHERE name = $1");
    match sqlx::query(&query).bind(params.name).execute(&pool.0).await {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
