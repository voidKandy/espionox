use std::fs;
use std::{fmt::Display, path::Path};
use tracing::{self, info};

use crate::agent::spo_agents::SummarizerAgent;

#[derive(Debug, Clone)]
pub struct File {
    pub filepath: Box<Path>,
    pub chunks: Vec<FileChunk>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    pub parent_filepath: Box<Path>,
    pub content: String,
    pub index: i16,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match &self.summary {
            None => format!(
                "FilePath: {}, Content: {}",
                &self.filepath.display(),
                &self.content()
            ),
            Some(summary) => format!(
                "FilePath: {}, Content: {}, Summary: {}",
                &self.filepath.display(),
                &self.content(),
                &summary
            ),
        };
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
            summary: None,
            // summary_embedding: Vec::new(),
        }
        .chunkify()
    }
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

    pub async fn get_summary(&mut self) {
        let summary = SummarizerAgent::summarize(self.to_owned())
            .await
            .expect("Failed to get summary");
        self.summary = Some(summary);
    }
}
