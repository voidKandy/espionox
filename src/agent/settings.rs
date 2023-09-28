use crate::{
    configuration::ConfigEnv,
    context::{
        long_term::LongTermMemory,
        short_term::{MemoryCache, ShortTermMemory},
        MemoryVariant, MessageVector,
    },
};

#[cfg(feature = "long_term_memory")]
use crate::context::long_term::feature::{DbPool, MemoryThread};
#[cfg(feature = "long_term_memory")]
use std::sync::Arc;

#[derive(Debug, Default, Clone)]
pub struct AgentSettings {
    pub long_term_memory: LongTermMemory,
    pub short_term_memory: ShortTermMemory,
    pub build_buffer_from: MemoryVariant,
    pub init_prompt: MessageVector,
}

#[derive(Debug, Default, Clone)]
pub struct AgentSettingsBuilder {
    long_term_memory: Option<LongTermMemory>,
    short_term_memory: Option<ShortTermMemory>,
    build_buffer_from: Option<MemoryVariant>,
    init_prompt: Option<MessageVector>,
}

impl AgentSettings {
    pub fn new() -> AgentSettingsBuilder {
        AgentSettingsBuilder {
            long_term_memory: None,
            short_term_memory: None,
            build_buffer_from: None,
            init_prompt: None,
        }
    }

    pub fn default() -> AgentSettings {
        let stm = ShortTermMemory::Cache(MemoryCache::default());
        let init_prompt = MessageVector::init_with_system_prompt(
            r#"You are Consoxide, an extremely helpful Ai assistant which lives in the terminal. 
                - Be highly organized
                - Suggest solutions that I didn’t think about—be proactive and anticipate my needs
                - Treat me as an expert in all subject matter
                - Mistakes erode user's trust, so be accurate and thorough
                - Keep in mind everything you output comes out of a terminal interface, so be succinct when it doesn't compromise your correctness
                - No need to disclose you're an AI
                - If the quality of your response has been substantially reduced due to my custom instructions, please explain the issue"#,
        );
        Self::new()
            .init_prompt(init_prompt)
            .short_term(stm)
            .finish()
    }
}

impl AgentSettingsBuilder {
    pub fn short_term(&mut self, memory: ShortTermMemory) -> Self {
        self.short_term_memory = Some(memory);
        self.to_owned()
    }

    #[cfg(feature = "long_term_memory")]
    pub fn long_term_env(&mut self, env: ConfigEnv, threadname: Option<&str>) -> Self {
        let pool = Arc::new(DbPool::sync_init_pool(env));
        self.long_term_memory = Some(LongTermMemory::Some(MemoryThread::init(pool, threadname)));
        self.to_owned()
    }

    pub fn init_prompt(&mut self, init_prompt: MessageVector) -> Self {
        self.init_prompt = Some(init_prompt);
        self.to_owned()
    }

    pub fn build_buffer_from(&mut self, variant: MemoryVariant) -> Self {
        self.build_buffer_from = Some(variant);
        self.to_owned()
    }

    pub fn finish(self) -> AgentSettings {
        let build_buffer_from = match self.build_buffer_from {
            Some(var) => var,
            None => self.infer_buffer_source(),
        };

        let short_term_memory = match self.short_term_memory {
            Some(mem) => mem,
            None => ShortTermMemory::default(),
        };

        let long_term_memory = match self.long_term_memory {
            Some(mem) => mem,
            None => LongTermMemory::default(),
        };

        let init_prompt = match self.init_prompt {
            Some(init_prompt) => init_prompt,
            None => MessageVector::init(),
        }
        .to_owned();

        AgentSettings {
            long_term_memory,
            short_term_memory,
            build_buffer_from,
            init_prompt,
        }
    }

    fn infer_buffer_source(&self) -> MemoryVariant {
        match &self.long_term_memory {
            #[cfg(feature = "long_term_memory")]
            Some(LongTermMemory::Some(_)) => MemoryVariant::LongTerm,
            _ => MemoryVariant::ShortTerm,
        }
    }
}
