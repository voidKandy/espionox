use super::test_agent;
use crate::{functions::weather_test_function, helpers::init_test};
#[allow(unused_imports)]
use espionoxide::{
    agent::Agent,
    context::{memory::Memory, Context},
    language_models::openai::functions::{CustomFunction, Property, PropertyInfo},
};

#[tokio::test]
async fn stream_completion_works() {
    crate::helpers::init_test();
    let mut agent = test_agent();
    let prompt =
        String::from("Hello chat agent, please respond with a long sentence on any subject");
    let mut receiver = agent.stream_prompt(&prompt).await;

    let timeout_duration = std::time::Duration::from_millis(100);

    while let Ok(result) = tokio::time::timeout(timeout_duration, receiver.receive())
        .await
        .unwrap()
    {
        match result {
            Some(response) => {
                tracing::info!("{}", response);
            }
            None => {
                tracing::warn!("Got None");
            }
        }
    }
}

#[ignore]
#[test]
fn function_agent_test() {
    init_test();
    let mut agent = test_agent();
    let response_json = agent.function_prompt(
        weather_test_function(),
        "What's the weather like in Detroit michigan in celcius?",
    );
    tracing::info!("Response json: {:?}", response_json);
    if let Some(location) = response_json
        .as_object()
        .and_then(|obj| obj.get("location"))
        .and_then(|value| value.as_str())
    {
        if location != "Detroit, MI" && location != "Detroit, Michigan" {
            assert!(false, "Location returned incorrectly")
        }
    }
    assert_eq!(
        response_json
            .as_object()
            .and_then(|obj| obj.get("unit"))
            .and_then(|value| value.as_str())
            .unwrap(),
        "celcius"
    );
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
    agent.switch_mem(espionoxide::context::MemoryVariant::new_short());

    let prompt = String::from("Hello chat agent");
    agent.context.buffer.push_std("user", &prompt);
    let cached_buf = agent.context.buffer.clone();

    agent.switch_mem(espionoxide::context::MemoryVariant::Forget);
    assert_ne!(cached_buf, agent.context.buffer);

    agent.switch_mem(espionoxide::context::MemoryVariant::new_short());
    assert_eq!(cached_buf, agent.context.buffer);
}
