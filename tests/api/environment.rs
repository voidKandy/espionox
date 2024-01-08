use std::time::Duration;

use crate::{helpers, init_test};
use espionox::{
    environment::{
        agent::{
            language_models::openai::gpt::streaming_utils::CompletionStreamStatus,
            memory::{messages::MessageRole, Message},
        },
        dispatch::{Dispatch, EnvNotification},
    },
    Agent,
};
use tokio;

#[tokio::test]
async fn insert_agent_works() {
    let agent = Agent::default();
    let mut environment = helpers::test_env();
    let handle = environment.insert_agent(None, agent).await;
    assert!(handle.is_ok());
}

#[tokio::test]
async fn io_prompt_agent_works() {
    init_test();
    let agent = Agent::default();
    let mut environment = helpers::test_env();
    let mut handle = environment
        .insert_agent(Some("jerry"), agent)
        .await
        .unwrap();

    environment.spawn().await.expect("Failed to spawn");

    let message = Message::new(MessageRole::User, "Hello!");
    let ticket = handle.request_io_completion(message).await.unwrap();

    environment.finalize_dispatch().await.unwrap();
    let noti: EnvNotification = environment.wait_for_notification(&ticket).await.unwrap();
    println!("Got noti: {:?}", noti);
    let message = match noti {
        EnvNotification::GotAssistantMessageResponse { message, .. } => message,
        _ => panic!("WRONG"),
    };

    assert_eq!(message.role, MessageRole::Assistant);
}

#[tokio::test]
async fn stream_prompt_agent_works() {
    init_test();
    let agent = Agent::default();
    let mut environment = helpers::test_env();
    let mut handle = environment
        .insert_agent(Some("jerry"), agent)
        .await
        .unwrap();

    environment.spawn().await.expect("Failed to spawn");

    let message = Message::new(MessageRole::User, "Hello!");
    let ticket = &handle
        .request_stream_completion(message.clone())
        .await
        .unwrap();
    let noti: EnvNotification = environment.wait_for_notification(ticket).await.unwrap();
    let mut handler = match noti {
        EnvNotification::GotStreamHandle { handler, .. } => handler,
        _ => panic!("WRONG"),
    };

    let mut whole_message = String::new();
    while let Ok(status) = handler
        .receive(&handle.id, environment.clone_sender())
        .await
    {
        match status {
            CompletionStreamStatus::Working(token) => {
                whole_message.push_str(&token);
                println!("{}", token);
            }
            CompletionStreamStatus::Finished => {
                break;
            }
        };
    }

    let mut stack = environment
        .take_notifications()
        .await
        .expect("Res stack is None");

    let stack = stack
        .take_by_agent(&handle.id)
        .expect("Failed to get stack of agent notis");
    println!("{:?}", stack);
    let cache_update = match stack.iter().nth(0).unwrap() {
        EnvNotification::ChangedCache { message, .. } => &message.content,
        _ => panic!("WRONG"),
    };
    environment.finalize_dispatch().await.unwrap();

    assert_eq!(&whole_message, cache_update);
}
