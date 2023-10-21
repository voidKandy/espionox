use espionox::{
    agents::Agent,
    configuration::ConfigEnv,
    language_models::{
        openai::gpt::{Gpt, GptConfig},
        LanguageModel,
    },
    memory::Memory,
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

pub fn test_env() -> ConfigEnv {
    ConfigEnv::new("testing")
}

pub fn test_gpt() -> Gpt {
    let config = GptConfig::init(test_env());
    let model_override = None;
    Gpt {
        config,
        model_override,
    }
}

pub fn test_agent_lt() -> Agent {
    let memory = Memory::build()
        .env(test_env())
        .long_term_thread("TestingThread")
        .finished();
    let model = LanguageModel::from(test_gpt());
    Agent { memory, model }
}

pub fn test_agent() -> Agent {
    let memory = Memory::build().finished();
    let model = LanguageModel::from(test_gpt());
    Agent { memory, model }
}
