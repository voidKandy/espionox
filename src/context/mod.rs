pub mod integrations;
pub mod memory;
pub mod messages;

pub use memory::*;
pub use messages::*;
use short_term::ShortTermMemory;

use self::integrations::core::BufferDisplay;
use crate::{agent::AgentSettings, errors::error_chain_fmt};
use long_term::LongTermMemory;

#[derive(thiserror::Error)]
pub enum ContextError {
    BuildError(String),
    Unexpected(#[from] anyhow::Error),
}

impl std::fmt::Debug for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    pub short_term: ShortTermMemory,
    pub long_term: LongTermMemory,
    buffer_source: MemoryVariant,
}

#[derive(Debug, Clone, Default)]
pub enum MemoryVariant {
    #[default]
    ShortTerm,
    LongTerm,
}

impl Context {
    pub fn from_settings(settings: AgentSettings) -> Context {
        let short_term = settings.short_term_memory;
        let long_term = settings.long_term_memory;
        let buffer_source = settings.build_buffer_from;
        Context {
            short_term,
            long_term,
            buffer_source,
        }
    }

    pub fn buffer(&self) -> MessageVector {
        match self.buffer_source {
            MemoryVariant::ShortTerm => self.short_term.load(),
            MemoryVariant::LongTerm => self.long_term.load(),
        }
    }

    pub fn switch_buffer_source(&mut self, source: MemoryVariant) {
        self.buffer_source = source;
    }

    pub fn push_to_buffer(&mut self, role: &str, displayable: &impl BufferDisplay) {
        match self.buffer_source {
            MemoryVariant::ShortTerm => self
                .short_term
                .save(MessageVector::new(displayable.buffer_display(role))),
            MemoryVariant::LongTerm => self
                .long_term
                .save(MessageVector::new(displayable.buffer_display(role))),
        }
    }

    pub fn save_buffer_to(&mut self, memory: MemoryVariant) {
        let buffer = self.buffer();
        let buf_ref_iter = buffer.as_ref().iter();
        let buf_difference = match memory {
            MemoryVariant::ShortTerm => MessageVector::from(
                buf_ref_iter
                    .filter(|&value| !self.short_term.load().as_ref().contains(value))
                    .cloned()
                    .collect::<Vec<Message>>(),
            ),
            MemoryVariant::LongTerm => MessageVector::from(
                buf_ref_iter
                    .filter(|&value| !self.long_term.load().as_ref().contains(value))
                    .cloned()
                    .collect::<Vec<Message>>(),
            ),
        };
        match memory {
            MemoryVariant::ShortTerm => {
                self.short_term.save(buf_difference);
            }
            MemoryVariant::LongTerm => {
                self.long_term.save(buf_difference);
            }
        }
    }
}
