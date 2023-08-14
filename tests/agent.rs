#[allow(unused_imports)]
use consoxide::{
    context::{memory::Memory, Context},
    handler::agent::Agent,
};

use consoxide::language_models::openai::functions::enums::FnEnum;

#[ignore]
#[test]
fn function_agent_test() {
    let mut agent = Agent::init();
    let prompt = String::from("[Investigate the failing test in src/tests/context.rs, Check the assertion at line 42 in src/tests/context.rs, Analyze the error message to understand the cause of the failure, Fix the failing test to pass the assertion]");
    agent.context.push_to_buffer("user", &prompt);

    let function = FnEnum::ExecuteGenerateRead.to_function();
    let response = agent.function_prompt(function);
    println!("{:?}", &response);
    assert!(true);
}

#[ignore]
#[test]
fn prompt_agent_test() {
    let mut agent = Agent::init();
    let prompt = String::from("Hello chat agent");
    let response = agent.prompt(&prompt);
    println!("{:?}", &response);
    assert!(true);
}

#[test]
fn to_and_from_short_term_test() {
    let mut agent = Agent::init();
    let prompt = String::from("Hello chat agent");
    agent.context.push_to_buffer("user", &prompt);
    let cached_buf = agent.context.buf_ref();

    agent.switch_mem(Memory::Forget);
    assert_ne!(cached_buf, agent.context.buf_ref());

    agent.switch_mem(Memory::Remember(
        consoxide::context::memory::LoadedMemory::Cache,
    ));
    assert_eq!(cached_buf, agent.context.buf_ref());
}
