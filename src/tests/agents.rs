#[allow(unused_imports)]
use crate::agent::handler::{AgentHandler, SpecialAgent};

#[ignore]
#[tokio::test]
async fn function_agent_test() {
    let mut handler = AgentHandler::new(SpecialAgent::IoAgent);
    let prompt = String::from("I need to make a shell file that opens a tmux session");
    handler.update_context("user", &prompt).unwrap();

    let function = &handler
        .special_agent
        .get_functions()
        .as_ref()
        .unwrap()
        // Get commands test
        .get(0)
        .unwrap()
        .to_function();
    let response = handler.function_prompt(function).await;
    assert!(response.is_ok());
}

#[ignore]
#[tokio::test]
async fn prompt_agent_test() {
    let mut handler = AgentHandler::new(SpecialAgent::ChatAgent);
    let prompt = String::from("Hello chat agent");
    handler.update_context("user", &prompt).unwrap();

    let response = handler.prompt().await;
    assert!(response.is_ok());
}

#[test]
fn update_agent_context_test() {
    let mut handler = AgentHandler::new(SpecialAgent::ChatAgent);
    let context_before = &handler.context.messages.clone();
    let prompt = String::from("Hello chat agent");
    handler.update_context("user", &prompt).unwrap();

    assert_ne!(context_before, &handler.context.messages);
}
