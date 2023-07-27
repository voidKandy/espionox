use super::super::init::DbPool;
use super::super::models::message::*;
use sqlx::postgres::PgQueryResult;

pub async fn get_messages(
    pool: &DbPool,
    params: GetMessageParams,
) -> anyhow::Result<Vec<MessageModelSql>> {
    let query = sqlx::query_as::<_, MessageModelSql>("SELECT * FROM messages WHERE thread_id = $1");

    match query.bind(params.thread_id).fetch_all(&pool.0).await {
        Ok(result) => Ok(result),
        Err(err) => Err(err.into()),
    }
}

pub async fn post_message(
    pool: &DbPool,
    message: CreateMessageBody,
) -> anyhow::Result<PgQueryResult> {
    let query = "INSERT INTO messages (id, thread_id, role, content) VALUES ($1, $2, $3, $4)";
    match sqlx::query(query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(message.thread_id)
        .bind(message.role)
        .bind(message.content)
        .execute(&pool.0)
        .await
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

pub async fn delete_message(
    pool: &DbPool,
    params: DeleteMessageParams,
) -> anyhow::Result<PgQueryResult> {
    let query = &format!("DELETE FROM messages WHERE id = $1");
    match sqlx::query(&query).bind(params.id).execute(&pool.0).await {
        Ok(rows) => Ok(rows),
        Err(err) => Err(err.into()),
    }
}
