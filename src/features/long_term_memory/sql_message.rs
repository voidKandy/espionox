use crate::memory::{message::MessageMetadata, traits::ToMessage, Message};
use anyhow::anyhow;

#[derive(sqlx::FromRow, Clone)]
pub struct SqlMessage<'a> {
    id: &'a str,
    role: &'a str,
    content: &'a str,
    metadata: MessageMetadata,
}

pub trait ToSqlMessage: ToMessage {
    fn to_sql_message(&self) -> SqlMessage;
}

impl<'a> TryInto<SqlMessage<'a>> for Message {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<SqlMessage<'a>, Self::Error> {
        match self {
            Message::Standard {
                id,
                role,
                content,
                metadata,
            } => {
                let role = &role.to_string();
                let content = &content;
                let metadata = metadata;
                Ok(SqlMessage {
                    id: &id,
                    role,
                    content,
                    metadata,
                })
            }
            _ => Err(anyhow!("Cannot build SqlMessage from non-standard message")),
        }
    }
}
