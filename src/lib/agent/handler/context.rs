use super::super::config::memory::Memory;
use crate::lib::io::{
    file_interface::{Directory, File},
    tmux::session::TmuxSession,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub messages: Vec<Value>,
    pub memory: Memory,
    pub session: TmuxSession,
}

pub trait Contextual {
    fn make_relevant(&self, context: &mut Context);
}

impl Context {
    pub fn new(messages: Vec<Value>, memory: Memory, session: TmuxSession) -> Context {
        Context {
            messages,
            memory,
            session,
        }
    }
    pub fn switch(&mut self, memory: Memory) {
        let new_self = match memory {
            Memory::ShortTerm => Context::new(
                memory.load_short_term().unwrap(),
                memory,
                TmuxSession::new(),
            ),
            _ => {
                if self.memory == Memory::ShortTerm {
                    self.memory.save_to_short_term(self.messages.to_owned());
                }
                memory.init()
            }
        };
        *self = new_self;
    }
    pub fn append_to_messages(&mut self, role: &str, content: &str) {
        self.messages
            .push(json!({"role": role, "content": content}));
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
        context.append_to_messages(
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
        context.append_to_messages(
            "system",
            &format!("Relavent Files: [{}]", payload.join(", ")),
        )
    }
}

impl Contextual for TmuxSession {
    fn make_relevant(&self, context: &mut Context) {
        context.append_to_messages(
            "system",
            &format!(
                "TmuxSession:\n watched_pane: {}\noutput_pane: {}
                    \n io: [{}]\n",
                self.watched_pane.name,
                self.output_pane.name,
                // self.pwd,
                self.io
                    .values()
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        )
    }
}
