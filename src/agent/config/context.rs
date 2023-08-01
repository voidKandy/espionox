use super::{super::config::memory::Memory, memory::LoadedMemory};
use crate::{
    core::{
        file_interface::{Directory, File},
        io::Io,
    },
    database::{api, handlers},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
    pub memory: Memory,
    pub buffer: Vec<Value>,
}

pub trait Contextual {
    fn messagize(&self) -> String;
}

impl Context {
    pub fn build(memory: Memory) -> Context {
        Context {
            buffer: memory.load(),
            memory,
        }
    }

    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer.push(json!({"role": role, "content": content}));
    }
    pub fn switch_mem(&mut self, memory: Memory) {
        self.memory.save(self.buffer.clone());
        *self = Context::build(memory);
    }
    pub fn remember_file(&self, file: File) {
        match &self.memory {
            Memory::Remember(LoadedMemory::LongTerm(threadname)) => {
                let sql_tup = api::sql_from_file(file, threadname);
                LoadedMemory::LongTerm(threadname.to_string()).store_file_tup(sql_tup);
            }
            _ => panic!("Memory not long term"),
        };
    }
}

impl Contextual for Directory {
    fn messagize(&self) -> String {
        let mut files_payload = vec![];
        self.files.iter().for_each(|f| {
            files_payload.push(match f.summary.as_str() {
                "" => format!(
                    "FilePath: {}, Content: {}",
                    &f.filepath.display(),
                    &f.content()
                ),
                _ => format!(
                    "FilePath: {}, Content: {}, Summary: {}",
                    &f.filepath.display(),
                    &f.content(),
                    &f.summary
                ),
            })
        });
        format!(
            "Relevant Directory path: {}, Child Directories: [{:?}], Files: [{}]",
            self.dirpath.display().to_string(),
            self.children
                .clone()
                .into_iter()
                .map(|c| c.dirpath.display().to_string())
                .collect::<Vec<String>>()
                .join(", "),
            files_payload.join(", ")
        )
    }
}

impl Contextual for File {
    fn messagize(&self) -> String {
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

impl Contextual for Io {
    fn messagize(&self) -> String {
        format!("in: {}\nout: {}", self.i, self.o)
    }
}
