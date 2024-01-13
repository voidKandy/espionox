use crate::memory::{
    embeddings::EmbeddingVector,
    message::{MessageMetadata, MetadataInfo},
    Message,
};
use anyhow::anyhow;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone)]
pub struct SqlMessage {
    id: String,
    role: String,
    content: String,
    metadata_id: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct SqlMetadata {
    pub id: String,
    pub content_embedding: Option<EmbeddingVector>,
}

#[derive(sqlx::FromRow, Clone)]
pub struct SqlInfo {
    parent_id: String,
    name: String,
    content: String,
    embedding: Option<EmbeddingVector>,
}

pub fn serialize_message(
    message: &Message,
) -> Result<(SqlMessage, SqlMetadata, Vec<SqlInfo>), anyhow::Error> {
    if let Message::Standard {
        id,
        role,
        content,
        metadata,
    } = message
    {
        let role = role.to_string();
        let (meta, infos) = serialize_metadata(metadata);

        let sql = SqlMessage {
            id: id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            metadata_id: meta.id,
        };
        Ok((sql, meta, infos))
    } else {
        Err(anyhow!("Cannot build SqlMessage from non-standard message"))
    }
}

fn serialize_metadata(data: &MessageMetadata) -> (SqlMetadata, Vec<SqlInfo>) {
    let id = Uuid::new_v4().to_string();
    let content_embedding = data.content_embedding;
    let infos = data.infos.iter().fold(vec![], |mut acc, info| {
        acc.push(serialize_info(info, &id));
        acc
    });
    let meta = SqlMetadata {
        id,
        content_embedding,
    };
    (meta, infos)
}

fn serialize_info(info: &MetadataInfo, parent_id: &str) -> SqlInfo {
    let parent_id = parent_id.to_string();
    SqlInfo {
        parent_id,
        name: info.name,
        content: info.content,
        embedding: info.embedding,
    }
}
