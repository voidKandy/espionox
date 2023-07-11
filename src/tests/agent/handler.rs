use crate::agent::functions::enums::FnEnum;
#[allow(unused_imports)]
use crate::agent::handler::AgentHandler;

#[ignore]
#[tokio::test]
async fn function_agent_test() {
    let mut handler = AgentHandler::new();
    let prompt = String::from("[Investigate the failing test in src/tests/context.rs, Check the assertion at line 42 in src/tests/context.rs, Analyze the error message to understand the cause of the failure, Fix the failing test to pass the assertion]");
    handler.context.append_to_messages("user", &prompt);

    let function = &FnEnum::ExecuteGenerateRead.to_function();
    let response = handler.function_prompt(function).await;
    println!("{:?}", &response);
    assert!(response.is_ok());
}

#[ignore]
#[tokio::test]
async fn watcher_agent_test() {
    let mut handler = AgentHandler::new();
    handler.monitor_user().await;
    assert!(false);
}

#[ignore]
#[tokio::test]
async fn prompt_agent_test() {
    let mut handler = AgentHandler::new();
    let prompt = String::from("Hello chat agent");
    handler.context.append_to_messages("user", &prompt);

    let response = handler.prompt().await;
    assert!(response.is_ok());
}

#[test]
fn update_agent_context_test() {
    let mut handler = AgentHandler::new();
    let context_before = &handler.context.current_messages().clone();
    let prompt = String::from("Hello chat agent");
    handler.context.append_to_messages("user", &prompt);

    assert_ne!(context_before, &handler.context.current_messages());
}
