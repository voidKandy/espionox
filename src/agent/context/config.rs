use super::tmux_session::Pane;
use super::walk::{Directory, File};
use serde_json::{json, Value};

pub struct Context {
    pub messages: Vec<Value>,
    pub files: Vec<File>,
    pub directories: Vec<Directory>,
    pub panes: Vec<Pane>,
    pub guidance: Option<String>,
}

impl Context {
    pub fn new(sys_prompt: Option<&str>) -> Context {
        let messages = match sys_prompt {
            Some(prompt) => {
                vec![(json!({"role": "system", "content": prompt}))]
            }
            None => {
                vec![]
            }
        };
        Context {
            messages,
            files: vec![],
            directories: vec![],
            panes: vec![],
            guidance: None,
        }
    }
    pub fn make_relevant(&mut self, dirs: Option<&Vec<Directory>>, files: Option<&Vec<File>>) {
        match dirs {
            Some(ds) => ds
                .into_iter()
                .for_each(|d| self.directories.push(d.clone())),
            None => {}
        };
        match files {
            Some(fs) => fs.into_iter().for_each(|f| self.files.push(f.clone())),
            None => {}
        }
    }
    pub fn append_to_messages(&mut self, role: &str, content: &str) {
        self.messages
            .push(json!({"role": role, "content": content}))
    }
    pub fn deliver_files_to_messages(&mut self, files: Vec<File>, message: &str) {
        let mut payload = vec![];
        files.iter().for_each(|f| {
            payload.push(format!(
                "FilePath: {}, Content: {}",
                &f.filepath.display(),
                &f.summary
            ));
        });
        self.append_to_messages(
            "system",
            &format!("{} Files: {}", message, payload.join(",")),
        )
    }
    pub fn refresh_pane(&mut self) {
        self.panes.push(Pane::capture(Some(25)))
    }
}
