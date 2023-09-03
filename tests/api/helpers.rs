use consoxide::{
    agent::{Agent, AgentSettings},
    context::{Memory, MessageVector},
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub fn init_test() {
    Lazy::force(&TRACING);
}

pub fn test_settings() -> AgentSettings {
    AgentSettings::new(
        Some(Memory::LongTerm("Testing_Thread".to_string())),
        MessageVector::new(vec![]),
    )
}

pub fn test_agent() -> Agent {
    Agent::build(test_settings()).expect("Failed to build test agent")
}
