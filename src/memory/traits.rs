use uuid::Uuid;

use super::{cache::Memory, error::MemoryError, message::*, MessageVector};
use crate::{core::*, language_models::embed, Agent};
use std::fmt::Display;

pub trait ToMessage: std::fmt::Debug + Display + ToString + Send + Sync {
    async fn to_message(&self) -> Result<Message, MemoryError>;
    async fn get_metadata(&self) -> Result<MessageMetadata, MemoryError> {
        Ok(MessageMetadata::empty())
    }
    fn role(&self) -> MessageRole;
}

pub trait ToMessageVector {
    async fn to_message_vector(&self) -> Result<MessageVector, MemoryError>;
}

pub async fn summarize_code_content(content: &str) -> Result<String, MemoryError> {
    let memory = Memory::new(
        crate::agents::utils::FILE_SUMMARIZER_SYS_PROMPT,
        super::long_term::LongTermMemory::None,
    );
    let mut a = Agent {
        memory,
        ..Default::default()
    };
    let message = Message::new_standard(MessageRole::System, content);
    a.prompt(message).await.map_err(|e| e.into())
}

impl ToMessage for String {
    async fn to_message(&self) -> Result<Message, MemoryError> {
        let message = Message::Standard {
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
        let message = Message::Standard {
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
        let chunk_summary = summarize_code_content(&self.content).await?;
        let chunk_parent_info = MetadataInfo::new(
            "parent_filepath",
            &self.parent_filepath.display().to_string(),
            false,
        );
        let chunk_summary_info = MetadataInfo::new("summary", &chunk_summary, true);
        let meta = MessageMetadata {
            content_embedding,
            infos: vec![chunk_parent_info, chunk_summary_info],
        };
        Ok(meta)
    }
}

impl ToMessage for Io {
    async fn to_message(&self) -> Result<Message, MemoryError> {
        let message = Message::Standard {
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
        let content = summarize_code_content(&self.content()).await?;
        let message = Message::Standard {
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
