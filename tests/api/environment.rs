use crate::{helpers, init_test};
use espionox::{
    environment::agent::memory::{messages::MessageRole, Message},
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
async fn prompt_agent_works() {
    init_test();
    let agent = Agent::default();
    let mut environment = helpers::test_env();
    let mut handle = environment
        .insert_agent(Some("jerry"), agent)
        .await
        .unwrap();

    environment.spawn().await.expect("Failed to spawn");

    let message = Message::new(MessageRole::User, "Hello!");
    handle.request_completion(message).await.unwrap();

    environment.finalize_dispatch().await.unwrap();
    let stack = environment.get_responses_stack().await;
    println!("Stack: {:?}", stack);
    let message = match stack.into_iter().next().unwrap() {
        espionox::environment::EnvResponse::ChangedCache { message, .. } => message,
    };

    assert_eq!(message.role, MessageRole::Assistant);
}

