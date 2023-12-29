use espionox::{
    agents::{
        spo_agents::{AgentObserver, ObservationProtocol},
        Agent,
    },
    configuration::ConfigEnv,
    core::File,
    language_models::{openai::gpt::Gpt, LanguageModel},
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

// pub fn test_env() -> ConfigEnv {
//     ConfigEnv::new("testing")
// }

pub fn test_gpt() -> Gpt {
    // let config = GptConfig::from(test_env());
    let api_key = "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".to_string();
    let mut gpt = Gpt::default();
    gpt.api_key = Some(api_key);
    gpt
}

#[cfg(feature = "long_term_memory")]
pub fn test_agent_lt() -> Agent {
    let memory = Memory::build()
        .env(test_env())
        .long_term_thread("TestingThread")
        .finished();
    let model = LanguageModel::from(test_gpt());
    Agent {
        memory,
        model,
        ..Default::default()
    }
}

pub fn test_agent() -> Agent {
    let memory = Memory::build().finished();
    let model = LanguageModel::from(test_gpt());
    Agent {
        memory,
        model,
        ..Default::default()
    }
}

pub fn observed_test_agent() -> Agent {
    let memory = Memory::build().finished();
    let model = LanguageModel::from(test_gpt());
    let mut protocol = ObservationProtocol::new();
    protocol.input_mutator(
        espionox::agents::spo_agents::observer::ObservationStep::BeforeAndAfterPrompt,
    );

    let observer = Some(AgentObserver::new(protocol));
    Agent {
        memory,
        model,
        observer,
    }
}

// pub fn test_file() -> File {
//     let filepath = test_env().config_file_path();
//     File::from(filepath)
// }
