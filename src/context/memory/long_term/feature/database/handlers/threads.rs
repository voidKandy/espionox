use super::super::init::DbPool;
use super::super::models::threads::*;
use sqlx::postgres::PgQueryResult;
use std::error::Error;

pub async fn get_all_threads(pool: &DbPool) -> Result<Vec<String>, Box<dyn Error>> {
    let threads: Vec<ThreadModelSql> =
        match sqlx::query_as::<_, ThreadModelSql>("SELECT * FROM threads")
            .fetch_all(pool.as_ref())
            .await
        {
            Ok(result) => result,
            Err(err) => panic!("ERROR GETTING ALL THREADS: {err:?}"),
        };
    Ok(threads.iter().map(|t| t.name.to_owned()).collect())
}

pub async fn get_thread(pool: &DbPool, name: &str) -> anyhow::Result<ThreadModelSql> {
    let query = format!("SELECT * FROM threads WHERE name = {}", name);
    match sqlx::query_as::<_, ThreadModelSql>(&query)
        .fetch_one(pool.as_ref())
        .await
    {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_thread(pool: &DbPool, name: &str) -> anyhow::Result<PgQueryResult> {
    let query = format!("INSERT INTO threads (name) VALUES ($1)");
    let res = sqlx::query(&query)
        .bind(name)
        .execute(pool.as_ref())
        .await?;
    Ok(res)
}

pub async fn delete_thread(pool: &DbPool, name: &str) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM threads WHERE name = $1");
    let res = sqlx::query(&query)
        .bind(name)
        .execute(pool.as_ref())
        .await?;
    Ok(res)
}
