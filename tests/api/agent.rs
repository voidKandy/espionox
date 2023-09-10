use super::test_agent;
use crate::helpers::init_test;
#[allow(unused_imports)]
use consoxide::{
    agent::Agent,
    context::{memory::Memory, Context},
    language_models::openai::functions::{CustomFunction, Property, PropertyInfo},
};
use serde_json::json;

pub fn weather_test_function() -> CustomFunction {
    let location_info = PropertyInfo::new(
        "description",
        json!("The city and state, e.g. San Francisco, CA"),
    );
    let unit_info = PropertyInfo::new("enum", json!(["celcius", "fahrenheit"]));

    let location_prop = Property::build_from("location")
        .return_type("string")
        .add_info(location_info)
        .finished();
    let unit_prop = Property::build_from("unit")
        .return_type("string")
        .add_info(unit_info)
        .finished();

    CustomFunction::build_from("get_current_weather")
        .description("Get the current weather in a given location")
        .add_property(location_prop, true)
        .add_property(unit_prop, false)
        .finished()
}

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
    agent.switch_mem(Memory::ShortTerm);

    let prompt = String::from("Hello chat agent");
    agent.context.push_to_buffer("user", &prompt);
    let cached_buf = agent.context.buffer.clone();

    agent.switch_mem(Memory::Forget);
    assert_ne!(cached_buf, agent.context.buffer);

    agent.switch_mem(Memory::ShortTerm);
    assert_eq!(cached_buf, agent.context.buffer);
}
