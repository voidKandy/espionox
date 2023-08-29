use consoxide::{
    context::{Memory, MessageVector},
    handler::{Agent, AgentSettings},
};

pub fn test_settings() -> AgentSettings {
    AgentSettings::new(
        Some(Memory::LongTerm("Testing_Thread".to_string())),
        MessageVector::new(vec![]),
    )
}

pub fn test_agent() -> Agent {
    Agent::build(test_settings()).expect("Failed to build test agent")
}
