use crate::{helpers, init_test};
use espionox::{
    environment::agent::{
        language_models::LanguageModel,
        memory::{messages::MessageRole, Message},
    },
    Agent,
};
use tokio;

#[tokio::test]
async fn insert_agent_works() {
    let agent = Agent::default();
    let id = agent.id.clone();
    let mut environment = helpers::test_env();
    environment.insert_agent(agent).await;
    assert!(environment.get_agent_handle(&id).await.is_some());
}

#[tokio::test]
async fn prompt_agent_works() {
    init_test();
    let agent = Agent::default();
    let id = agent.id.clone();
    let mut environment = helpers::test_env();
    environment.insert_agent(agent).await;
    environment.spawn().await.expect("Failed to spawn");
    let mut handle = environment
        .get_agent_handle(&id)
        .await
        .expect("Failed to get handle?");
    let message = Message::new(
        espionox::environment::agent::memory::messages::MessageRole::User,
        "Hello!",
    );
    handle.request_completion(message).await.unwrap();

    environment.finalize_dispatch().await.unwrap();
    let stack = environment.get_responses_stack().await;
    println!("Stack: {:?}", stack);
    let message = match stack.into_iter().next().unwrap() {
        espionox::environment::EnvResponse::ChangedCache { message, .. } => message,
    };

    assert_eq!(message.role, MessageRole::Assistant);
}
