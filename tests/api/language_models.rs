use espionox::{
    agents::{actions::io_completion, Agent},
    language_models::completions::CompletionModel,
};
use serde_json::json;

use crate::init_test;

#[tokio::test]
async fn failed_request_does_not_overflow_stack() {
    init_test();
    let llm = CompletionModel::default_anthropic("invalid_key");
    let mut a = Agent::new(None, llm);

    let res = io_completion(&mut a, ()).await;

    println!("{:?}", res);
    assert!(res.is_err());
}
