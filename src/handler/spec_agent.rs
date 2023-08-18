use super::agent::Agent;
use crate::context::{Memory, MessageVector};

pub enum SpecialAgent {
    Watcher(MessageVector),
}

impl SpecialAgent {
    pub fn init(self) -> Agent {
        match self {
            SpecialAgent::Watcher(init_prompt) => {
                let mut agent = Agent::init(Memory::LongTerm("Watcher_Agent_Thread".to_string()));
                if agent.context.buffer.len() == 0 {
                    for p in init_prompt.as_ref() {
                        agent.context.push_to_buffer(p.role(), p.content());
                    }
                }
                agent
            }
        }
    }
}
