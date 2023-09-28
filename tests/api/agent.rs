use super::test_agent;
use crate::{functions::weather_test_function, helpers::init_test};
#[allow(unused_imports)]
use espionox::{
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
    let mut receiver = agent
        .stream_prompt(prompt)
        .await
        .expect("Failed to get stream receiver");

    let timeout_duration = std::time::Duration::from_millis(250);

    while let Ok(Some(result)) = tokio::time::timeout(timeout_duration, receiver.receive())
        .await
        .unwrap()
    {
        tracing::info!("{}", result);
    }
}

#[test]
fn prompting_can_be_blocked_on_a_tokio_runtime() {
    let test = std::thread::spawn(|| {
        crate::helpers::init_test();
        let mut agent = test_agent();

        let response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(agent.prompt(String::from("Hello")));
        assert!(response.is_ok());
    })
    .join();
    assert!(test.is_ok());
}

#[ignore]
#[tokio::test]
async fn function_agent_test() {
    init_test();
    let mut agent = test_agent();
    let response_json = agent
        .function_prompt(
            weather_test_function(),
            "What's the weather like in Detroit michigan in celcius?".to_string(),
        )
        .await
        .expect("Failed to get function response");
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
#[tokio::test]
async fn prompt_agent_test() {
    let mut agent = test_agent();
    let prompt = String::from("Hello chat agent");
    let response = agent.prompt(prompt).await.expect("Failed to get response");
    println!("{:?}", &response);
    assert!(true);
}
