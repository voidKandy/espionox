#![allow(unused)]
use super::models;
use super::models::{ContextModelSql, FileModelSql};
use dotenv::dotenv;
use sqlx::postgres::PgQueryResult;
use sqlx::FromRow;
use sqlx::Row;
use sqlx::{Connection, Execute};
use std::env;
use std::error::Error;

pub struct DbPool(sqlx::PgPool);

impl DbPool {
    pub async fn init() -> DbPool {
        dotenv().ok();
        let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("{:?}", &url);
        match sqlx::postgres::PgPool::connect(&url).await {
            Ok(pool) => DbPool(pool),
            Err(err) => panic!("Error initializing DB pool: {err:?}"),
        }
    }

    pub async fn get_context(
        &self,
        params: models::ContextParams,
    ) -> anyhow::Result<ContextModelSql> {
        match sqlx::query_as!(
            ContextModelSql,
            "SELECT * FROM contexts WHERE name = $1",
            params.name
        )
        .fetch_one(&self.0)
        .await
        {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn post_context(
        &self,
        params: models::ContextParams,
    ) -> anyhow::Result<PgQueryResult> {
        let query = format!("INSERT INTO contexts (id, name) VALUES ($1, $2)");
        match sqlx::query(&query)
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(params.name)
            .execute(&self.0)
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn delete_context(
        &self,
        params: models::ContextParams,
    ) -> anyhow::Result<PgQueryResult> {
        let query = &format!("DELETE FROM contexts WHERE name = $1");
        match sqlx::query(&query).bind(params.name).execute(&self.0).await {
            Ok(rows) => Ok(rows),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn get_file(&self, params: models::GetFileParams) -> anyhow::Result<FileModelSql> {
        let query = sqlx::query_as::<_, FileModelSql>("SELECT * FROM files WHERE filepath = $1");

        match query.bind(params.filepath).fetch_one(&self.0).await {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn post_file(&self, file: models::CreateFileBody) -> anyhow::Result<PgQueryResult> {
        let query = "INSERT INTO files (id, context_id, filepath, parent_dir_path, summary, summary_embedding) VALUES ($1, $2, $3, $4, $5, $6)";
        match sqlx::query(query)
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(file.context_id)
            .bind(file.filepath)
            .bind(file.parent_dir_path)
            .bind(file.summary)
            .bind(file.summary_embedding)
            .execute(&self.0)
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn delete_file(
        &self,
        params: models::DeleteFileParams,
    ) -> anyhow::Result<PgQueryResult> {
        let query = &format!("DELETE FROM files WHERE filepath = $1");
        match sqlx::query(&query)
            .bind(params.filepath)
            .execute(&self.0)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(err.into()),
        }
    }
}
