use crate::{functions::test_function, init_test, test_anthropic_agent, test_openai_agent};
use espionox::{
    agents::{
        actions::{function_completion, io_completion, stream_completion},
        listeners::ListenerTrigger,
        memory::Message,
    },
    language_models::completions::streaming::ProviderStreamHandler,
};
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
    let function = test_function();
    let message = Message::new_user("What's the weather like in Detroit michigan in celcius?");
    a.cache.push(message);

    let json: Value = a
        .do_action(
            function_completion,
            (function, "get_current_weather"),
            Option::<ListenerTrigger>::None,
        )
        .await
        .unwrap();

    if let Some(location) = json
        .as_object()
        .and_then(|obj| obj.get("location"))
        .and_then(|value| value.as_str())
    {
        if location != "Detroit, MI" && location != "Detroit, Michigan" {
            assert!(false, "Location returned incorrectly")
        }
    }
    assert_eq!(
        json.as_object()
            .and_then(|obj| obj.get("unit"))
            .and_then(|value| value.as_str())
            .unwrap(),
        "celcius"
    );
}
