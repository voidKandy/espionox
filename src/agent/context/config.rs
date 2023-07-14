use super::tmux_session::TmuxSession;
use super::walk::{Directory, File};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::io::prelude::*;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Context {
    pub messages: Vec<Value>,
    pub memory: Memory,
    pub session: TmuxSession,
}

pub trait Contextual {
    fn make_relevant(&self, context: &mut Context);
}

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Memory {
    LongTerm,
    ShortTerm,
    Temporary,
}

impl Memory {
    pub const SHORT_TERM_PATH: &str = "./src/agent/context/short_term_memory.json";
    pub fn init(self) -> Context {
        match self {
            Memory::LongTerm => {
                Context::new(self.load_long_term().unwrap(), self, TmuxSession::new())
            }
            _ => Context::new(vec![], self, TmuxSession::new()),
        }
    }

    pub fn load_long_term(&self) -> Result<Vec<Value>, Box<dyn Error>> {
        unimplemented!();
    }

    pub fn load_short_term(&self) -> Result<Vec<Value>, Box<dyn Error>> {
        let mut contents = String::new();
        fs::File::open(Self::SHORT_TERM_PATH)?.read_to_string(&mut contents)?;
        println!("{contents}");
        match serde_json::from_str(&contents) {
            Ok(Value::Array(array)) => {
                return Ok(array);
            }
            Err(err) => {
                return Err(format!("Problem getting Json from String: {err:?}").into());
            }
            Ok(data) => Ok(vec![data]),
        }
    }

    pub fn save_to_short_term(&self, content: Vec<Value>) -> bool {
        fs::write(
            Self::SHORT_TERM_PATH,
            format!(
                "[{}]",
                content
                    .iter()
                    .map(|m| m.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
        )
        .unwrap();
        true
    }
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
                "TmuxSession:\n watched_pane: {}, present directory: {}, contents: [{}]",
                self.watched_pane,
                self.pwd,
                self.contents
                    .values()
                    .into_iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        )
    }
}
