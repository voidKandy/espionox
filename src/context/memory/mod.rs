#[cfg(feature = "long_term_memory")]
pub mod long_term;
pub mod short_term;
use super::MessageVector;

pub trait Memory: std::fmt::Debug {
    fn load(&self) -> MessageVector;
    fn save(&self, messages: MessageVector);
}
