use super::{
    super::config::memory::Memory,
    memory::LoadedMemory::{Cache, LongTerm},
};
use crate::core::{
    file_interface::{Directory, File},
    io::Io,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Serialize, Deserialize)]
pub struct Context {
    pub memory: Memory,
    pub buffer: Vec<Value>,
}

pub trait Contextual {
    fn make_relevant(&self, context: &mut Context);
}

impl Context {
    pub fn build(memory: &Memory) -> Context {
        Context {
            buffer: memory.load(),
            memory: memory.to_owned(),
        }
    }

    pub fn push_to_buffer(&mut self, role: &str, content: &str) {
        self.buffer.push(json!({"role": role, "content": content}));
    }
    pub fn switch_mem(&mut self, memory: Memory) {
        self.memory.save(&self.buffer);
        *self = Context::build(&memory);
    }
}

impl Contextual for Directory {
    fn make_relevant(&self, context: &mut Context) {
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
        self.children.iter().for_each(|d| {
            d.make_relevant(context);
        });
        context.push_to_buffer(
            "system",
            &format!(
                "Relevant Directory path: {}, Child Directories: [{:?}], Files: [{}]",
                self.dirpath.display().to_string(),
                self.children
                    .clone()
                    .into_iter()
                    .map(|c| c.dirpath.display().to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                files_payload.join(", ")
            ),
        )
    }
}

impl Contextual for Vec<File> {
    fn make_relevant(&self, context: &mut Context) {
        let mut payload = vec![];
        self.iter().for_each(|f| {
            payload.push(match f.summary.as_str() {
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
        context.push_to_buffer(
            "system",
            &format!("Relavent Files: [{}]", payload.join(", ")),
        )
    }
}

impl Contextual for Vec<Io> {
    fn make_relevant(&self, context: &mut Context) {
        let mut message = String::new();
        self.iter()
            .for_each(|io| message.push_str(&format!("in: {}\nout: {}", io.i, io.o)));
        context.push_to_buffer("system", &message)
    }
}
