use super::tmux_session::Pane;
use super::walk::{Directory, File};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Context {
    pub messages: HashMap<String, Vec<Value>>,
    current_conversation: String,
    pub pane: Pane,
}

pub trait Contextual {
    fn make_relevant(&self, context: &mut Context) {}
}

impl Context {
    pub fn new(name: &str, sys_prompt: Option<&str>) -> Context {
        let messages = match sys_prompt {
            Some(prompt) => {
                let mut map = HashMap::new();
                map.insert(
                    name.to_string(),
                    vec![(json!({"role": "system", "content": prompt}))],
                );
                map
            }
            None => {
                let mut map = HashMap::new();
                map.insert(name.to_string(), vec![]);
                map
            }
        };
        Context {
            messages,
            current_conversation: name.to_string(),
            pane: Pane::new(),
        }
    }
    pub fn change_conversation(&mut self, name: &str) {
        if self.messages.keys().all(|k| k != name) {
            self.messages.insert(name.to_string(), vec![]);
        };
        self.current_conversation = name.to_string();
    }
    pub fn drop_conversation(&mut self) {
        self.messages.remove(&self.current_conversation);
        let to_convo = self.messages.keys().nth(0).unwrap().to_owned();
        self.change_conversation(&to_convo);
    }
    pub fn append_to_messages(&mut self, role: &str, content: &str) {
        self.messages
            .entry(self.current_conversation.clone())
            .and_modify(|c| c.push(json!({"role": role, "content": content})));
    }
    pub fn current_messages(&self) -> Vec<Value> {
        self.messages
            .get(&self.current_conversation)
            .unwrap()
            .to_vec()
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
                    &f.content
                ),
                _ => format!(
                    "FilePath: {}, Content: {}, Summary: {}",
                    &f.filepath.display(),
                    &f.content,
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
                    &f.content
                ),
                _ => format!(
                    "FilePath: {}, Content: {}, Summary: {}",
                    &f.filepath.display(),
                    &f.content,
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

impl Contextual for Pane {
    fn make_relevant(&self, context: &mut Context) {
        context.append_to_messages(
            "system",
            &format!(
                "Tmux-Pane:\n name: {}, present directory: {}, contents: [{}]",
                self.name,
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
