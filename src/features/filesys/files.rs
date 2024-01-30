use crate::environment::agent::{language_models::embed, memory::*};
use std::fs;
use std::path::PathBuf;
use std::{fmt::Display, path::Path};
use tracing::{self, info};

#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub filepath: Box<Path>,
    pub chunks: Vec<FileChunk>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileChunk {
    pub parent_filepath: Box<Path>,
    pub content: String,
    pub index: i16,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!(
            "FilePath: {}, Content: {}",
            &self.filepath.display(),
            &self.content()
        );
        write!(f, "{}", string)
    }
}

impl Display for FileChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!(
            "FilePath: {}, ChunkIndex: {}, Content: {}",
            &self.parent_filepath.display(),
            &self.index,
            &self.content,
        );
        write!(f, "{}", string)
    }
}

impl From<&str> for File {
    fn from(filename: &str) -> Self {
        let filepath = fs::canonicalize(Path::new(filename)).unwrap().into();
        File {
            filepath,
            chunks: vec![],
        }
        .chunkify()
    }
}

impl From<PathBuf> for File {
    fn from(path: PathBuf) -> Self {
        File {
            filepath: path.into(),
            chunks: vec![],
        }
        .chunkify()
    }
}

impl ToMessage for File {
    fn to_message(&self) -> Message {
        // let content = summarize_code_content(&self.content());
        let message = Message {
            role: self.role(),
            content: "hardcoded content".to_string(),
        };
        message
    }

    fn role(&self) -> MessageRole {
        MessageRole::System
    }

    //     fn get_metadata(&self) -> MessageMetadata {
    //         let content_embedding = embed(&self.content()).ok().map(|emb| emb.into());
    //         let meta = MessageMetadata {
    //             content_embedding,
    //             infos: vec![],
    //         };
    //         meta
    //     }
}

impl ToMessageVector for File {
    fn to_message_vector(&self) -> MessageVector {
        let mut mvec = MessageVector::init();
        for chunk in self.chunks.iter() {
            mvec.push(chunk.to_message());
        }
        mvec
    }
}

impl ToMessage for FileChunk {
    fn to_message(&self) -> Message {
        let content = &self.content;
        let content = content.to_string();
        let message = Message {
            role: self.role(),
            content,
        };
        message
    }

    fn role(&self) -> MessageRole {
        MessageRole::System
    }

    //     fn get_metadata(&self) -> MessageMetadata {
    //         let content_embedding = embed(&self.content).ok().map(|emb| emb.into());
    //         // let chunk_summary = summarize_code_content(&self.content);
    //         let chunk_parent_info = MetadataEmbedding::new(
    //             "parent_filepath",
    //             &self.parent_filepath.display().to_string(),
    //             false,
    //         );
    //         let chunk_summary_info = MetadataEmbedding::new("summary", "summary hardcoded", true);
    //         let meta = MessageMetadata {
    //             content_embedding,
    //             infos: vec![chunk_parent_info, chunk_summary_info],
    //         };
    //         meta
    //     }
}

impl File {
    #[tracing::instrument]
    pub fn chunkify(&mut self) -> Self {
        info!("chunkifying: {}", &self.filepath.to_str().unwrap());
        let content = fs::read_to_string(&self.filepath).unwrap_or_else(|e| {
            println!(
                "Failed to get content of {}\nError: {e:?}",
                &self.filepath.to_str().unwrap()
            );
            e.to_string()
        });
        let lines: Vec<&str> = content.lines().collect();
        lines.chunks(50).enumerate().for_each(|(i, c)| {
            self.chunks.push(FileChunk {
                parent_filepath: self.filepath.clone(),
                content: c.join("\n"),
                // content_embedding: Vec::new(),
                index: i as i16,
            });
        });
        self.to_owned()
    }

    pub fn content(&self) -> String {
        let mut content: Vec<&str> = Vec::new();
        self.chunks.iter().for_each(|c| content.push(&c.content));
        content.join("\n")
    }
}
