use super::BufferDisplay;
use std::fs;
use std::path::Path;
use tracing::{self, info};

#[derive(Debug, Clone)]
pub struct File {
    pub filepath: Box<Path>,
    pub chunks: Vec<FileChunk>,
    pub summary: String,
    pub summary_embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct FileChunk {
    pub parent_filepath: Box<Path>,
    pub content: String,
    pub content_embedding: Vec<f32>,
    pub index: i16,
}

impl File {
    pub fn build(filename: &str) -> File {
        let filepath = fs::canonicalize(Path::new(filename)).unwrap().into();
        File {
            filepath,
            chunks: vec![],
            summary: String::new(),
            summary_embedding: Vec::new(),
        }
        .chunkify()
    }

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
                content_embedding: Vec::new(),
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

impl BufferDisplay for File {
    fn buffer_display(&self) -> String {
        match self.summary.as_str() {
            "" => format!(
                "FilePath: {}, Content: {}",
                &self.filepath.display(),
                &self.content()
            ),
            _ => format!(
                "FilePath: {}, Content: {}, Summary: {}",
                &self.filepath.display(),
                &self.content(),
                &self.summary
            ),
        }
    }
}

impl BufferDisplay for FileChunk {
    fn buffer_display(&self) -> String {
        format!(
            "FilePath: {}, ChunkIndex: {}, Content: {}",
            &self.parent_filepath.display(),
            &self.index,
            &self.content,
        )
    }
}
