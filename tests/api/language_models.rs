use crate::init_test;
use espionox::{agents::Agent, language_models::completions::CompletionModel};

#[tokio::test]
async fn failed_request_does_not_overflow_stack() {
    init_test();
    let llm = CompletionModel::default_anthropic("invalid_key");
    let mut a = Agent::new(None, llm);

    let res = a.io_completion().await;

    println!("{:?}", res);
    assert!(res.is_err());
}
