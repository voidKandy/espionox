use crate::{helpers, init_test};
use espionox::{
    environment::agent::{language_models::LanguageModel, memory::Message},
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
    environment.run().await;
    let mut handle = environment
        .get_agent_handle(&id)
        .await
        .expect("Failed to get handle?");
    let message = Message::new(
        espionox::environment::agent::memory::messages::MessageRole::User,
        "Hello!",
    );
    handle.request_completion(message).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let stack = environment.get_event_stack().await;
    println!("Stack: {:?}", stack);
    assert!(false);
}
