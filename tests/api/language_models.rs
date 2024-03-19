use std::any::Any;

use espionox::{
    agents::{
        memory::{Message, MessageStack},
        Agent,
    },
    environment::agent_handle::InferenceEndpointHandler,
    language_models::{
        anthropic::{self, AnthropicCompletionHandler},
        inference::LLMEmbeddingHandler,
        openai::{completions::OpenAiCompletionHandler, embeddings::OpenAiEmbeddingModel},
        LLM,
    },
};
use reqwest::Client;

use crate::init_test;

#[ignore]
#[tokio::test]
async fn embedding_handlers_get_embeddings() {
    init_test();
    dotenv::dotenv().ok();
    let text = "Heyyyy this is a test";
    let api_key = std::env::var("OPENAI_KEY").unwrap();
    let client = reqwest::Client::new();
    let embedder =
        LLM::new_embedding_model(LLMEmbeddingHandler::from(OpenAiEmbeddingModel::Small), None);
    let response = embedder.get_embedding(text, &api_key, &client).await;
    assert!(response.is_ok())
}

#[ignore]
#[tokio::test]
async fn completion_handlers_get_completions() {
    init_test();
    dotenv::dotenv().ok();
    let client = Client::new();

    let open_ai_key = std::env::var("OPENAI_KEY").unwrap();
    let anth_key = std::env::var("ANTHROPIC_KEY").unwrap();
    let openai = LLM::default_openai();
    let anthropic = LLM::default_anthropic();

    let mut messages = MessageStack::new("You are a test model");
    messages.push(Message::new_user("HELLO"));

    let oai_res = openai
        .get_io_completion(&messages, &open_ai_key, &client)
        .await;
    let anth_res = anthropic
        .get_io_completion(&messages, &anth_key, &client)
        .await;
    assert!(oai_res.is_ok() && anth_res.is_ok());
}
