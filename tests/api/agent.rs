use crate::{init_test, test_anthropic_agent, test_openai_agent};
use espionox::{
    agents::{
        actions::{function_completion, io_completion, stream_completion},
        listeners::ListenerTrigger,
        memory::Message,
    },
    language_models::completions::{functions::Function, streaming::ProviderStreamHandler},
};
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use tracing::info;

#[ignore]
#[tokio::test]
async fn io_prompt_agent_works() {
    init_test();
    let mut a = test_openai_agent();
    a.cache.push(Message::new_user("Hello!"));
    let result = a
        .do_action(io_completion, (), Option::<ListenerTrigger>::None)
        .await;
    info!("response: {:?}", result);
    assert!(result.is_ok());

    let mut a = test_anthropic_agent();
    a.cache.push(Message::new_user("Hello!"));
    let result = a
        .do_action(io_completion, (), Option::<ListenerTrigger>::None)
        .await;
    info!("response: {:?}", result);
    assert!(result.is_ok())
}

#[ignore]
#[tokio::test]
async fn stream_prompt_agent_works() {
    init_test();
    let mut a = test_openai_agent();
    a.cache.push(Message::new_user("Hello!"));

    let mut response: ProviderStreamHandler = a
        .do_action(stream_completion, (), Option::<ListenerTrigger>::None)
        .await
        .unwrap();
    while let Some(res) = response.receive(&mut a).await {
        info!("OpenAi Token: {:?}", res)
    }
    assert_eq!(a.cache.len(), 3);

    let mut a = test_anthropic_agent();
    a.cache.push(Message::new_user("Hello!"));

    let mut response: ProviderStreamHandler = a
        .do_action(stream_completion, (), Option::<ListenerTrigger>::None)
        .await
        .unwrap();
    while let Some(res) = response.receive(&mut a).await {
        info!("Anthropic token: {:?}", res)
    }
    info!("CACHE: {:?}", a.cache);
    assert_eq!(a.cache.len(), 3)
}

#[ignore]
#[tokio::test]
async fn function_prompt_agent_works() {
    init_test();
    let mut a = test_openai_agent();
    let test_func_str = r#"get_n_day_weather_forecast(location: string, format!: enum('celcius' | 'farenheight'), num_days!: integer)
        where 
            i am 'get an n-day weather forecast'
            location is 'the city and state, e.g. san francisco, ca'
            format is 'the temperature unit to use. infer this from the users location.'
            num_days is 'the number of days to forcast'
        "#;
    let function = Function::try_from(test_func_str).unwrap();
    let message = Message::new_user("What's the weather like in Detroit michigan in celcius?");
    a.cache.push(message);

    let json: Value = a
        .do_action(
            function_completion,
            function,
            Option::<ListenerTrigger>::None,
        )
        .await
        .unwrap();
    // let json: Ret = serde_json::from_str(json).unwrap();
    info!("test got json: {:?}", json);
    assert_eq!(
        json.get("format").and_then(|value| value.as_str()).unwrap(),
        "celcius"
    );
    if let Some(location) = json.get("location").and_then(|value| value.as_str()) {
        if location != "Detroit, MI" && location != "Detroit, Michigan" {
            assert!(false, "Location returned incorrectly")
        }
    }
}
