use crate::{functions::weather_test_function, helpers, init_test};
use espionox::{
    agents::{
        memory::{messages::MessageRole, Message},
        Agent,
    },
    environment::{
        dispatch::{EnvNotification, ThreadSafeStreamCompletionHandler},
        notification_stack::NotificationStack,
    },
    language_models::{
        anthropic::AnthropicCompletionHandler,
        openai::completions::{streaming::CompletionStreamStatus, OpenAiCompletionHandler},
        LLM,
    },
};
use serde_json::Value;
use tokio;

#[tokio::test]
async fn insert_agent_works() {
    let agent = Agent::new("test", LLM::default_openai());
    let mut environment = helpers::test_env();
    let handle = environment.insert_agent(None, agent).await;
    assert!(handle.is_ok());
}

#[ignore]
#[tokio::test]
async fn io_prompt_agent_works() {
    init_test();
    let agent = Agent::new("test", LLM::default_anthropic());
    let mut environment = helpers::test_env_with_keys();
    let mut a_handle = environment
        .insert_agent(Some("jerry"), agent)
        .await
        .unwrap();

    let mut env_handle = environment.spawn_handle().expect("Failed to spawn");

    let message = Message::new_user("Hello!");
    let ticket = a_handle.request_io_completion(message).await.unwrap();
    let noti: EnvNotification = env_handle.wait_for_notification(&ticket).await.unwrap();
    let message: &Message = noti.extract_body().try_into().unwrap();

    assert_eq!(message.role, MessageRole::Assistant);
}

#[ignore]
#[tokio::test]
async fn stream_prompt_agent_works() {
    init_test();
    let agent = Agent::new("test", LLM::default_openai());
    let mut environment = helpers::test_env_with_keys();
    let mut handle = environment
        .insert_agent(Some("jerry"), agent)
        .await
        .unwrap();

    let mut env_handle = environment.spawn_handle().expect("Failed to spawn");

    let message = Message::new_user("Hello!");
    let ticket = &handle
        .request_stream_completion(message.clone())
        .await
        .unwrap();
    tracing::error!("TEST GOT TICKET: {}", ticket);
    let noti: EnvNotification = env_handle.wait_for_notification(&ticket).await.unwrap();
    tracing::error!("TEST GOT NOTI: {:?}", noti);
    let handler: &ThreadSafeStreamCompletionHandler = noti.extract_body().try_into().unwrap();
    let mut handler = handler.lock().await;

    let mut whole_message = String::new();
    while let Some(CompletionStreamStatus::Working(token)) =
        handler.receive(&handle.id, env_handle.new_sender()).await
    {
        tracing::info!("TEST LOOPING");
        whole_message.push_str(&token);
        println!("{}", token);
    }
    tracing::info!("TEST GOT WHOLE MESSAGE: {}", whole_message);

    let mut stack = env_handle
        .finish_current_job()
        .await
        .expect("Couldn't finish thread");

    let stack = stack
        .take_by_agent(&handle.id)
        .expect("Failed to get stack of agent notis");
    println!("{:?}", stack);
}

#[ignore]
#[tokio::test]
async fn function_prompt_agent_works() {
    init_test();
    let agent = Agent::new("test", LLM::default_openai());
    let mut environment = helpers::test_env_with_keys();
    let mut a_handle = environment
        .insert_agent(Some("fn jerry"), agent)
        .await
        .unwrap();
    let function = weather_test_function();
    let message = Message::new_user("What's the weather like in Detroit michigan in celcius?");

    let mut env_handle = environment.spawn_handle().expect("Failed to spawn");
    let ticket = a_handle
        .request_function_prompt(function, message)
        .await
        .unwrap();

    let mut stack = env_handle.finish_current_job().await.unwrap();

    let noti: EnvNotification = stack.take_by_ticket(ticket).unwrap();
    println!("Got noti: {:?}", noti);
    let json: &Value = noti.extract_body().try_into().unwrap();
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
