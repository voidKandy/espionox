#[allow(unused_imports)]
use consoxide::{
    agent::Agent,
    context::{memory::Memory, Context},
    language_models::openai::functions::enums::FnEnum,
};

use super::test_agent;

#[tokio::test]
async fn stream_completion_works() {
    let mut agent = test_agent();
    let prompt = String::from("Hello chat agent");
    let mut receiver = agent.stream_prompt(&prompt).await;

    let timeout_duration = std::time::Duration::from_millis(200);

    while let Some(result) = tokio::time::timeout(timeout_duration, receiver.recv())
        .await
        .unwrap()
    {
        match result {
            Ok(response) => {
                println!("{}", response);
            }
            Err(err) => {
                panic!("Error: {:?}", err);
            }
        }
    }
}

#[ignore]
#[test]
fn function_agent_test() {
    let mut agent = test_agent();
    let prompt = String::from("[Investigate the failing test in src/tests/context.rs, Check the assertion at line 42 in src/tests/context.rs, Analyze the error message to understand the cause of the failure, Fix the failing test to pass the assertion]");
    agent.context.push_to_buffer("user", &prompt);

    let function = FnEnum::ExecuteGenerateRead.into();
    let response = agent.function_prompt(function);
    println!("{:?}", &response);
    assert!(true);
}

#[ignore]
#[test]
fn prompt_agent_test() {
    let mut agent = test_agent();
    let prompt = String::from("Hello chat agent");
    let response = agent.prompt(&prompt);
    println!("{:?}", &response);
    assert!(true);
}

#[test]
fn to_and_from_short_term_test() {
    let mut agent = test_agent();
    agent.switch_mem(Memory::ShortTerm);

    let prompt = String::from("Hello chat agent");
    agent.context.push_to_buffer("user", &prompt);
    let cached_buf = agent.context.buffer.clone();

    agent.switch_mem(Memory::Forget);
    assert_ne!(cached_buf, agent.context.buffer);

    agent.switch_mem(Memory::ShortTerm);
    assert_eq!(cached_buf, agent.context.buffer);
}
