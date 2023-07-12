use super::tmux_session::Pane;
use super::walk::{Directory, File};
use serde_json::{json, Value};

#[derive(Clone, Eq, PartialEq)]
pub struct Conversation {
    pub messages: Vec<Value>,
    pub context: Box<Context>,
    pub pane: Pane,
}

pub trait Contextual {
    fn make_relevant(&self, context: &mut Conversation) {}
}

#[derive(Clone, Eq, PartialEq)]
pub enum Context {
    // LongTerm,
    ShortTerm(Option<Box<Conversation>>),
    Temporary,
}

impl Context {
    pub fn init(self) -> Box<Conversation> {
        match self {
            // _ => self.load(),
            Context::ShortTerm(Some(conversation)) => conversation,
            Context::ShortTerm(None) => Conversation::new(vec![], self, Pane::new()),
            Context::Temporary => Conversation::new(vec![], self, Pane::new()),
        }
    }
}

impl Conversation {
    pub fn new(messages: Vec<Value>, context: Context, pane: Pane) -> Box<Conversation> {
        Box::new(Conversation {
            messages,
            context: Box::new(context),
            pane,
        })
    }
    pub fn switch(&mut self, context: Context) {
        *self = *context.init();
    }
    // pub fn forget_about(f: fn(Conversation) -> <T>&mut self, ) ->  {
    //     // if self.messages.keys().all(|k| k != name) {
    //     //     self.messages.insert(name.to_string(), vec![]);
    //     // };
    //     // self.current_conversation = name.to_string();
    // }
    // pub fn drop_conversation(&mut self) {
    //     self.messages.remove(&self.current_conversation);
    //     let to_convo = self.messages.keys().nth(0).unwrap().to_owned();
    //     self.change_conversation(&to_convo);
    // }
    pub fn append_to_messages(&mut self, role: &str, content: &str) {
        self.messages
            .push(json!({"role": role, "content": content}));
    }
}

impl Contextual for Directory {
    fn make_relevant(&self, context: &mut Conversation) {
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
    fn make_relevant(&self, context: &mut Conversation) {
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

impl Contextual for Pane {
    fn make_relevant(&self, context: &mut Conversation) {
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
