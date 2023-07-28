use super::super::init::DbPool;
use super::super::models::threads::*;
use sqlx::postgres::PgQueryResult;
use std::error::Error;

pub async fn get_all_threads(pool: &DbPool) -> Result<Vec<String>, Box<dyn Error>> {
    let threads: Vec<ThreadModelSql> =
        match sqlx::query_as!(ThreadModelSql, "SELECT * FROM threads",)
            .fetch_all(&pool.0)
            .await
        {
            Ok(result) => result,
            Err(err) => panic!("ERROR GETTING ALL THREADS: {err:?}"),
        };
    Ok(threads.iter().map(|t| t.name.to_owned()).collect())
}

pub async fn get_thread(pool: &DbPool, name: &str) -> anyhow::Result<ThreadModelSql> {
    match sqlx::query_as!(
        ThreadModelSql,
        "SELECT * FROM threads WHERE name = $1",
        name
    )
    .fetch_one(&pool.0)
    .await
    {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_thread(pool: &DbPool, name: &str) -> anyhow::Result<PgQueryResult> {
    let query = format!("INSERT INTO threads (id, name) VALUES ($1, $2)");
    match sqlx::query(&query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(name)
        .execute(&pool.0)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_thread(pool: &DbPool, name: &str) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM threads WHERE name = $1");
    match sqlx::query(&query).bind(name).execute(&pool.0).await {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
