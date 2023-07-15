use crate::lib::agent::config::{context::Context, memory::Memory};
use crate::lib::agent::functions::enums::FnEnum;
#[allow(unused_imports)]
use crate::lib::agent::handler::AgentHandler;

#[ignore]
#[tokio::test]
async fn function_agent_test() {
    let mut handler = AgentHandler::new(Memory::Temporary);
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
    let mut handler = AgentHandler::new(Memory::ShortTerm);
    handler.monitor_user().await;
    assert!(false);
}

#[ignore]
#[tokio::test]
async fn prompt_agent_test() {
    let mut handler = AgentHandler::new(Memory::ShortTerm);
    let prompt = String::from("Hello chat agent");
    handler.context.append_to_messages("user", &prompt);

    let response = handler.prompt().await;
    assert!(response.is_ok());
    handler.context.append_to_messages("user", "cool response");
    let response = handler.prompt().await;
    handler
        .context
        .memory
        .save_to_short_term(handler.context.messages);
    assert!(response.is_ok());
}

// #[test]
// fn to_and_from_short_term_test() {
//     let mut handler = AgentHandler::new(Memory::ShortTerm(None));
//     let prompt = String::from("Hello chat agent");
//     handler.context.append_to_messages("user", &prompt);
//     let short_term_mem = Box::new(&handler.context.clone());
//
//     handler.context.switch(Memory::Temporary);
//     assert_ne!(*short_term_mem.messages, &handler.context.messages);
//     handler
//         .context
//         .switch(Memory::ShortTerm(Some(short_term_mem)))
// }
