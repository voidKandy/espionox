use dotenv::dotenv;
use espionox::{
    agents::{
        actions::{function_completion, get_embedding, io_completion, stream_completion},
        listeners::ListenerTrigger,
        memory::Message,
        Agent,
    },
    language_models::{
        inference::LLMEmbeddingHandler,
        openai::{
            completions::streaming::StreamedCompletionHandler, embeddings::OpenAiEmbeddingModel,
        },
        LLM,
    },
};
use serde_json::Value;

use crate::{functions::weather_test_function, init_test};

fn test_agent() -> Agent {
    dotenv().ok();
    let openai_api_key = std::env::var("OPENAI_KEY").unwrap();
    let llm = LLM::default_openai(&openai_api_key);

    Agent::new(Some("I am running tests, say hello"), llm)
}

fn test_embedding_agent() -> Agent {
    dotenv().ok();
    let openai_api_key = std::env::var("OPENAI_KEY").unwrap();
    let llm = LLM::new_embedding_model(
        LLMEmbeddingHandler::OpenAi(OpenAiEmbeddingModel::Ada),
        None,
        &openai_api_key,
    );

    Agent::new(None, llm)
}

#[tokio::test]
async fn io_prompt_agent_works() {
    init_test();
    let mut a = test_agent();
    a.cache.push(Message::new_user("Hello!"));
    let result: String = a
        .do_action(io_completion, (), Option::<ListenerTrigger>::None)
        .await
        .unwrap();
    println!("response: {}", result);
}

#[tokio::test]
async fn stream_prompt_agent_works() {
    init_test();
    let mut a = test_agent();
    a.cache.push(Message::new_user("Hello!"));

    let mut response: StreamedCompletionHandler = a
        .do_action(stream_completion, (), Option::<ListenerTrigger>::None)
        .await
        .unwrap();
    while let Some(res) = response.receive(&mut a).await {
        println!("{:?}", res)
    }
    assert_eq!(a.cache.len(), 3)
}

#[tokio::test]
async fn function_prompt_agent_works() {
    init_test();
    let mut a = test_agent();
    let function = weather_test_function();
    let message = Message::new_user("What's the weather like in Detroit michigan in celcius?");
    a.cache.push(message);

    let json: Value = a
        .do_action(
            function_completion,
            function,
            Option::<ListenerTrigger>::None,
        )
        .await
        .unwrap();

    if let Some(location) = json
        .as_object()
        .and_then(|obj| obj.get("location"))
        .and_then(|value| value.as_str())
    {
        if location != "Detroit, MI" && location != "Detroit, Michigan" {
            assert!(false, "Location returned incorrectly")
        }
    }
    assert_eq!(
        json.as_object()
            .and_then(|obj| obj.get("unit"))
            .and_then(|value| value.as_str())
            .unwrap(),
        "celcius"
    );
}

#[tokio::test]
async fn get_embedding_agent_works() {
    init_test();
    let mut a = test_embedding_agent();
    let response: Vec<f32> = a
        .do_action(get_embedding, "embed this", Option::<ListenerTrigger>::None)
        .await
        .unwrap();
    println!("{:?}", response.len());
}
