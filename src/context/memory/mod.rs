pub mod long_term;
pub mod short_term;
use super::MessageVector;

pub trait Memory: std::fmt::Debug {
    fn load(&self) -> MessageVector;
    fn save(&mut self, messages: MessageVector);
}
