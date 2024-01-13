pub mod file;
pub mod file_chunks;
pub mod messages;
pub mod threads;

use crate::memory::ToMessage;

#[derive(sqlx::FromRow, Clone)]
pub struct SqlMessage<'a> {
    id: &'a str,
    role: Option<&'a str>,
    content: &'a str,
    metadata: SqlMessageMetadata<'a>,
}

#[derive(sqlx::FromRow, Clone)]
pub struct SqlMessageMetadata<'a> {
    model_generated_content: Option<&'a str>,
}

pub trait ToSqlMessage: ToMessage {
    fn to_sql_message(&self) -> SqlMessage;
}
