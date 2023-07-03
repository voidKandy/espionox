#[allow(unused_imports)]
use crate::agent::{agents::SpecialAgent, handler::AgentHandler};

#[ignore]
#[tokio::test]
async fn function_agent_test() {
    let mut handler = AgentHandler::new(SpecialAgent::WatcherAgent);
    let prompt = String::from("I need to make a shell file that opens a tmux session");
    handler.context.append_to_messages("user", &prompt);

    let function = &handler
        .special_agent
        .get_functions()
        .as_ref()
        .unwrap()
        .get(0)
        .unwrap()
        .to_function();
    let response = handler.function_prompt(function).await;
    println!("{:?}", &response);
    assert!(response.is_ok());
}

#[ignore]
#[tokio::test]
async fn prompt_agent_test() {
    let mut handler = AgentHandler::new(SpecialAgent::ChatAgent);
    let prompt = String::from("Hello chat agent");
    handler.context.append_to_messages("user", &prompt);

    let response = handler.prompt().await;
    assert!(response.is_ok());
}

#[test]
fn update_agent_context_test() {
    let mut handler = AgentHandler::new(SpecialAgent::ChatAgent);
    let context_before = &handler.context.messages.clone();
    let prompt = String::from("Hello chat agent");
    handler.context.append_to_messages("user", &prompt);

    assert_ne!(context_before, &handler.context.messages);
}

// #[test]
// fn watcher_test() {
//     let mut handler = AgentHandler::new(SpecialAgent::WatcherAgent);
//     loop {
//         handler.context.refresh_pane()
//     }
// }
