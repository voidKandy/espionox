use super::models;
use super::models::{ContextModelSql, ErrorModelSql, FileChunkModelSql, FileModelSql};
use dotenv::dotenv;
use sqlx::postgres::PgQueryResult;
use std::env;

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

    // ------ CONTEXTS ------ //
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

    // ------ FILES ------ //
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

    // ------ FILECHUNKS ------ //
    pub async fn get_file_chunks(
        &self,
        params: models::GetFileChunkParams,
    ) -> anyhow::Result<Vec<FileChunkModelSql>> {
        let query = sqlx::query_as::<_, FileChunkModelSql>(
            "SELECT * FROM file_chunks WHERE parent_file_id = $1",
        );

        match query.bind(params.parent_file_id).fetch_all(&self.0).await {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn post_file_chunk(
        &self,
        chunk: models::CreateFileChunkBody,
    ) -> anyhow::Result<PgQueryResult> {
        let query = "INSERT INTO file_chunks (id, parent_file_id, idx, content, content_embedding) VALUES ($1, $2, $3, $4, $5)";
        match sqlx::query(query)
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(chunk.parent_file_id)
            .bind(chunk.idx)
            .bind(chunk.content)
            .bind(chunk.content_embedding)
            .execute(&self.0)
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn delete_file_chunk(
        &self,
        params: models::DeleteFileChunkParams,
    ) -> anyhow::Result<PgQueryResult> {
        let query = &format!("DELETE FROM file_chunks WHERE parent_file_id = $1 AND idx = $2");
        match sqlx::query(&query)
            .bind(params.parent_file_id)
            .bind(params.idx)
            .execute(&self.0)
            .await
        {
            Ok(rows) => Ok(rows),
            Err(err) => Err(err.into()),
        }
    }

    // ------ ERRORS ------ //
    pub async fn get_errors(
        &self,
        params: models::GetErrorParams,
    ) -> anyhow::Result<Vec<ErrorModelSql>> {
        let query =
            sqlx::query_as::<_, ErrorModelSql>("SELECT * FROM errors WHERE context_id = $1");

        match query.bind(params.context_id).fetch_all(&self.0).await {
            Ok(result) => Ok(result),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn post_error(
        &self,
        chunk: models::CreateErrorBody,
    ) -> anyhow::Result<PgQueryResult> {
        let query = "INSERT INTO errors (id, context_id, content, content_embedding) VALUES ($1, $2, $3, $4)";
        match sqlx::query(query)
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(chunk.context_id)
            .bind(chunk.content)
            .bind(chunk.content_embedding)
            .execute(&self.0)
            .await
        {
            Ok(res) => Ok(res),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn delete_error(
        &self,
        params: models::DeleteErrorParams,
    ) -> anyhow::Result<PgQueryResult> {
        let query = &format!("DELETE FROM errors WHERE id = $1");
        match sqlx::query(&query).bind(params.id).execute(&self.0).await {
            Ok(rows) => Ok(rows),
            Err(err) => Err(err.into()),
        }
    }
}
