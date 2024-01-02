use uuid::Uuid;

use super::{
    super::super::errors::MemoryError, super::language_models::embed, super::Agent, messages::*,
    MessageVector,
};
use crate::{core::*, environment::Environment};
use std::fmt::Display;

pub trait ToMessage: std::fmt::Debug + Display + ToString + Send + Sync {
    async fn to_message(&self) -> Result<Message, MemoryError>;
    async fn get_metadata(&self) -> Result<MessageMetadata, MemoryError> {
        Ok(MessageMetadata::default())
    }
    fn role(&self) -> MessageRole;
}

pub trait ToMessageVector {
    async fn to_message_vector(&self) -> Result<MessageVector, MemoryError>;
}

// pub async fn summarize_code_content(content: &str) -> Result<String, MemoryError> {
//     // let env = Environment::new(None, api_key)
//     let cache = MessageVector::new(crate::environment::agent::utils::FILE_SUMMARIZER_SYS_PROMPT);
//     let mut a = Agent {
//         cache,
//         ..Default::default()
//     };
//     let message = Message::new(MessageRole::System, content);
//     a.prompt(message).await.map_err(|e| e.into())
// }

impl ToMessage for String {
    async fn to_message(&self) -> Result<Message, MemoryError> {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            role: self.role(),
            content: self.to_string(),
            metadata: self.get_metadata().await?,
        };
        Ok(message)
    }
    fn role(&self) -> MessageRole {
        MessageRole::System
    }
}

impl ToMessage for FileChunk {
    async fn to_message(&self) -> Result<Message, MemoryError> {
        let content = &self.content;
        let content = content.to_string();
        let message = Message {
            id: Uuid::new_v4().to_string(),
            role: self.role(),
            content,
            metadata: self.get_metadata().await?,
        };
        Ok(message)
    }

    fn role(&self) -> MessageRole {
        MessageRole::System
    }

    async fn get_metadata(&self) -> Result<MessageMetadata, MemoryError> {
        let content_embedding = embed(&self.content).ok().map(|emb| emb.into());
        // let chunk_summary = summarize_code_content(&self.content).await?;
        let chunk_parent_info = MetadataInfo::new(
            "parent_filepath",
            &self.parent_filepath.display().to_string(),
            false,
        );
        let chunk_summary_info = MetadataInfo::new("summary", "summary hardcoded", true);
        let meta = MessageMetadata {
            content_embedding,
            infos: vec![chunk_parent_info, chunk_summary_info],
        };
        Ok(meta)
    }
}

impl ToMessage for Io {
    async fn to_message(&self) -> Result<Message, MemoryError> {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            role: self.role(),
            content: self.to_string(),
            metadata: self.get_metadata().await?,
        };
        Ok(message)
    }

    fn role(&self) -> MessageRole {
        MessageRole::Other("io".to_string())
    }
}

impl ToMessage for File {
    async fn to_message(&self) -> Result<Message, MemoryError> {
        // let content = summarize_code_content(&self.content()).await?;
        let message = Message {
            id: Uuid::new_v4().to_string(),
            role: self.role(),
            content: "hardcoded content".to_string(),
            metadata: self.get_metadata().await?,
        };
        Ok(message)
    }

    fn role(&self) -> MessageRole {
        MessageRole::System
    }

    async fn get_metadata(&self) -> Result<MessageMetadata, MemoryError> {
        let content_embedding = embed(&self.content()).ok().map(|emb| emb.into());
        let meta = MessageMetadata {
            content_embedding,
            infos: vec![],
        };
        Ok(meta)
    }
}

impl ToMessageVector for File {
    async fn to_message_vector(&self) -> Result<MessageVector, MemoryError> {
        let mut mvec = MessageVector::init();
        for chunk in self.chunks.iter() {
            mvec.push(chunk.to_message().await?);
        }
        Ok(mvec)
    }
}

impl ToMessageVector for Directory {
    async fn to_message_vector(&self) -> Result<MessageVector, MemoryError> {
        let mut mvec = MessageVector::init();
        for file in self.files.iter() {
            mvec.append(file.to_message_vector().await?);
        }
        Ok(mvec)
    }
}
