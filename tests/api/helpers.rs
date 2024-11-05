use dotenv::dotenv;
use espionox::{
    agents::Agent,
    language_models::completions::CompletionModel,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use serde_json::{json, Value};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("ESPX_LOG").is_ok() {
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

pub fn test_openai_agent() -> Agent {
    dotenv().ok();
    let api_key = std::env::var("OPENAI_KEY").unwrap();
    let llm = CompletionModel::default_openai(&api_key);

    Agent::new(Some("I am running tests, say hello"), llm)
}

pub fn test_anthropic_agent() -> Agent {
    dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_KEY").unwrap();
    let llm = CompletionModel::default_anthropic(&api_key);

    Agent::new(Some("I am running tests, say hello"), llm)
}
// pub fn test_embedding_agent() -> Agent {
//     dotenv().ok();
//     let openai_api_key = std::env::var("OPENAI_KEY").unwrap();
//     let llm = LLM::new_embedding_model(
//         LLMEmbeddingHandler::OpenAi(OpenAiEmbeddingModel::Ada),
//         None,
//         &openai_api_key,
//     );
//     Agent::new(None, llm)
// }
pub fn test_function() -> Value {
    json!({
         "name": "get_current_weather",
              "description": "Get the current weather in a given location",
              "parameters": {
                "type": "object",
                "properties": {
                  "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                  },
                  "unit": {
                    "type": "string",
                    "enum": ["celcius", "fahrenheit"]
                  }
                },
                "required": ["location"]
        }
    })
}
