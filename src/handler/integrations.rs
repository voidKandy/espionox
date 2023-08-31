use super::{Agent, AgentSettings};
use crate::core::*;

#[derive(Debug)]
pub struct SummarizerAgent(Agent);

impl SummarizerAgent {
    #[tracing::instrument(name = "Create Summarizer Agent")]
    pub fn init() -> Self {
        SummarizerAgent(
            Agent::build(AgentSettings::summarizer())
                .expect("Failed to initialize summarizer agent"),
        )
    }

    #[tracing::instrument(name = "Summarize any struct that implements BufferDisplay")]
    pub fn summarize(&mut self, content: &mut impl BufferDisplay) -> String {
        self.0.switch_mem(crate::context::Memory::Forget);
        self.0.prompt(&content.buffer_display())
    }
}

pub trait BufferDisplay: std::fmt::Debug {
    fn buffer_display(&self) -> String;
}

impl BufferDisplay for File {
    fn buffer_display(&self) -> String {
        match &self.summary {
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

impl BufferDisplay for Directory {
    fn buffer_display(&self) -> String {
        let mut payload = String::new();
        for dir in self.children.iter() {
            let mut files_payload = vec![];
            dir.files.iter().for_each(|f| {
                files_payload.push(match &f.summary {
                    None => format!(
                        "FilePath: {}, Content: {}",
                        &f.filepath.display(),
                        &f.content()
                    ),
                    Some(summary) => format!(
                        "FilePath: {}, Content: {}, Summary: {}",
                        &f.filepath.display(),
                        &f.content(),
                        &summary
                    ),
                })
            });
            let dir_payload = format!(
                "Directory path: {}, Child Directories: [{:?}], Files: [{}]",
                dir.dirpath.display().to_string(),
                dir.children
                    .clone()
                    .into_iter()
                    .map(|c| c.dirpath.display().to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                files_payload.join(", ")
            );
            payload = format!("{}, {}", payload, dir_payload);
        }
        payload
    }
}

impl BufferDisplay for Io {
    fn buffer_display(&self) -> String {
        format!("Input: {}, Output: {}", &self.i, &self.o,)
    }
}
