#[allow(unused_imports)]
use consoxide::agent::{
    config::{context::Context, memory::Memory},
    handler::Agent,
};

use consoxide::language_models::openai::functions::enums::FnEnum;

#[ignore]
#[tokio::test]
async fn function_agent_test() {
    let mut agent = Agent::new(Memory::Forget);
    let prompt = String::from("[Investigate the failing test in src/tests/context.rs, Check the assertion at line 42 in src/tests/context.rs, Analyze the error message to understand the cause of the failure, Fix the failing test to pass the assertion]");
    agent.context.push_to_buffer("user", &prompt);

    let function = &FnEnum::ExecuteGenerateRead.to_function();
    let response = agent.function_prompt(function).await;
    println!("{:?}", &response);
    assert!(response.is_ok());
}
//
// #[ignore]
// #[tokio::test]
// async fn watcher_agent_test() {
//     let mut agent = Agentagent::new(Memory::ShortTerm);
//     agent.monitor_user().await;
//     assert!(false);
// }
//
#[ignore]
#[tokio::test]
async fn prompt_agent_test() {
    let mut agent = Agent::new(Memory::Forget);
    let prompt = String::from("Hello chat agent");
    agent.context.push_to_buffer("user", &prompt);

    let response = agent.prompt().await;
    assert!(response.is_ok());
    agent.context.push_to_buffer("user", "cool response");
    let response = agent.prompt().await;
    agent.context.memory.save(&agent.context.buffer);
    assert!(response.is_ok());
}

// #[test]
// fn to_and_from_short_term_test() {
//     let mut agent = Agentagent::new(Memory::ShortTerm(None));
//     let prompt = String::from("Hello chat agent");
//     agent.context.push_to_buffer("user", &prompt);
//     let short_term_mem = Box::new(&agent.context.clone());
//
//     agent.context.switch(Memory::Temporary);
//     assert_ne!(*short_term_mem.messages, &agent.context.messages);
//     agent
//         .context
//         .switch(Memory::ShortTerm(Some(short_term_mem)))
// }
