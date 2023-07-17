#![allow(unused)]
use super::models;
use super::models::ContextModelSql;
use dotenv::dotenv;
use sqlx::postgres::PgQueryResult;
use sqlx::Connection;
use sqlx::FromRow;
use sqlx::Row;
use std::env;
use std::error::Error;

pub async fn create_pool() -> sqlx::Result<sqlx::PgPool> {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("{:?}", &url);
    sqlx::postgres::PgPool::connect(&url).await
}

pub async fn get_context(
    params: models::ContextParams,
    pool: &sqlx::PgPool,
) -> anyhow::Result<ContextModelSql> {
    match sqlx::query_as!(
        ContextModelSql,
        "SELECT * FROM contexts WHERE name = $1",
        params.name
    )
    .fetch_one(pool)
    .await
    {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_context(
    params: models::ContextParams,
    pool: &sqlx::PgPool,
) -> anyhow::Result<PgQueryResult> {
    let query = format!("INSERT INTO contexts (id, name) VALUES ($1, $2)");
    match sqlx::query(&query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(params.name)
        .execute(pool)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_context(
    params: models::ContextParams,
    pool: &sqlx::PgPool,
) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM contexts WHERE name = $1");
    match sqlx::query(&query).bind(params.name).execute(pool).await {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_file(
    file: models::InsertFileBody,
    pool: &sqlx::PgPool,
) -> anyhow::Result<PgQueryResult> {
    let query = "INSERT INTO files (id, filepath, parent_dir_path, summary, summary_embedding) VALUES ($1, $2, $3, $4)";
    match sqlx::query(query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(file.filepath)
        .bind(file.parent_dir_path)
        .bind(file.summary)
        .bind(file.summary_embedding)
        .execute(pool)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}
