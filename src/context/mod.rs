pub mod integrations;
pub mod memory;
//
// pub use memory::*;
// pub use messages::*;
// use short_term::ShortTermMemory;
//
// use self::{integrations::core::ToMessage, short_term::MemoryCache};
// use crate::{agent::AgentSettings, errors::error_chain_fmt};
// use long_term::LongTermMemory;
//
// #[derive(thiserror::Error)]
// pub enum ContextError {
//     BuildError(String),
//     Unexpected(#[from] anyhow::Error),
// }
//
// impl std::fmt::Debug for ContextError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         error_chain_fmt(self, f)
//     }
// }
//
// impl std::fmt::Display for ContextError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
//
// #[derive(Clone, Debug)]
// pub struct Memory {
//     pub short_term: MemoryCache,
//     pub long_term: LongTermMemory,
//     // buffer_source: MemoryVariant,
// }
//
// #[derive(Debug, Clone, Default)]
// pub enum MemoryVariant {
//     #[default]
//     ShortTerm,
//     LongTerm,
// }
//
// impl Memory {
//     pub fn from_settings(settings: AgentSettings) -> Memory {
//         let short_term = settings.short_term_memory;
//         let long_term = settings.long_term_memory;
//         let buffer_source = settings.build_buffer_from;
//         Memory {
//             short_term,
//             long_term,
//             // buffer_source,
//         }
//     }
//
//     pub fn buffer(&self) -> MessageVector {
//         self.short_term.cache
//     }
//
//     // pub fn switch_buffer_source(&mut self, source: MemoryVariant) {
//     //     self.buffer_source = source;
//     // }
//
//     #[tracing::instrument(name = "Update current context buffer")]
//     pub fn push_to_buffer(&mut self, role: &str, displayable: impl ToMessage) {
//         self.short_term.cache.push_std(displayable.to_message(role));
//     }
//
//     pub fn save_buffer_to(&mut self, memory: MemoryVariant) {
//         let buffer = self.buffer();
//         let buf_ref_iter = buffer.as_ref().iter();
//         let buf_difference = match memory {
//             MemoryVariant::ShortTerm => MessageVector::from(
//                 buf_ref_iter
//                     .filter(|&value| !self.short_term.load().as_ref().contains(value))
//                     .cloned()
//                     .collect::<Vec<Message>>(),
//             ),
//             MemoryVariant::LongTerm => MessageVector::from(
//                 buf_ref_iter
//                     .filter(|&value| !self.long_term.load().as_ref().contains(value))
//                     .cloned()
//                     .collect::<Vec<Message>>(),
//             ),
//         };
//         match memory {
//             MemoryVariant::ShortTerm => {
//                 self.short_term.save(buf_difference);
//             }
//             MemoryVariant::LongTerm => {
//                 self.long_term.save(buf_difference);
//             }
//         }
//     }
// }
