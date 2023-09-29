use crate::{context::Message, core::*};
use std::fmt::Display;

pub trait BufferDisplay: std::fmt::Debug + Display + ToString {
    fn buffer_display(&self, role: &str) -> Message {
        Message::new_standard(role, &format!("{}", self.to_string().replace('\n', "")))
    }
}
impl BufferDisplay for String {}
impl BufferDisplay for str {}

impl BufferDisplay for FileChunk {}
impl BufferDisplay for File {}
impl BufferDisplay for Directory {}
impl BufferDisplay for Io {}
