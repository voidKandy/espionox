use std::collections::HashMap;

use espionox::{
    environment::{agent_handle::EndpointCompletionHandler, Environment},
    language_models::ModelProvider,
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

pub fn test_env<H: EndpointCompletionHandler>() -> Environment<H> {
    dotenv::dotenv().ok();
    Environment::new(Some("testing"), HashMap::new())
}

pub fn test_env_with_keys<H: EndpointCompletionHandler>() -> Environment<H> {
    dotenv::dotenv().ok();
    let api_key = std::env::var("OPENAI_KEY").unwrap();
    let mut keys = HashMap::new();
    keys.insert(ModelProvider::OpenAi, api_key);
    let api_key = std::env::var("ANTHROPIC_KEY").unwrap();
    keys.insert(ModelProvider::Anthropic, api_key);
    Environment::new(Some("testing"), keys)
}
