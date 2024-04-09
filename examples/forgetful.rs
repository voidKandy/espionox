use espionox::{
    agents::{
        memory::{Message, MessageStack},
        Agent,
    },
    environment::{
        agent_handle::MessageRole,
        dispatch::{
            listeners::ListenerMethodReturn, Dispatch, EnvListener, EnvMessage, EnvNotification,
            EnvRequest,
        },
        Environment,
    },
    language_models::{ModelProvider, LLM},
};
use std::collections::HashMap;
use uuid::Uuid;

/// This is a simple listener that will always ensure a model's memory never has anything more than
/// it's system prompt in it's memory. Useful for internal Summarizer agents
#[derive(Debug)]
pub struct Forgetful {
    watched_agent_id: String,
}

impl Forgetful {
    fn new(wa_id: &str) -> Self {
        Self {
            watched_agent_id: wa_id.to_owned(),
        }
    }
}

impl EnvListener for Forgetful {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        // After a completion is gotten from a model, Espionox implicitly requests a cache push of the
        // completion response as an Assistant message. By making the trigger on every requested push
        // of a message with a role of `MessageRole::Assistant`, we can wipe the agent's memory after
        // every completion response is gotten.
        if let EnvMessage::Request(req) = env_message {
            if let EnvRequest::PushToCache { agent_id, message } = req {
                if agent_id == &self.watched_agent_id
                    && message.role.actual() == &MessageRole::Assistant
                {
                    return Some(env_message);
                }
            }
        }
        None
    }

    fn method<'l>(
        &'l mut self,
        _trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            let watched_agent = dispatch
                .get_agent_mut(&self.watched_agent_id)
                .expect("Failed to get watched agent");
            // In order to wipe an agent's memory while still retaining system messages, we simply call
            // the `mut_filter_by` method on it's `cache` field.
            watched_agent.cache.mut_filter_by(MessageRole::System, true);

            // Once the agent's memory has been changed, we'll replace it's trigger with a
            // AgentStateUpdate message. This way the cache change is on record, and the assistant
            // message that triggered this listener won't be pushed to the agent's cache.
            let new_message = EnvNotification::AgentStateUpdate {
                ticket: Uuid::new_v4(),
                agent_id: self.watched_agent_id.to_owned(),
                cache: watched_agent.cache.clone(),
            };

            Ok(new_message.into())
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("OPENAI_KEY").unwrap();

    // Standard boilerplate for building an Environment & Agent
    let mut map = HashMap::new();
    map.insert(ModelProvider::OpenAi, api_key);
    let mut env = Environment::new(Some("testing"), map);
    let agent = Agent::new(Some("You are jerry!!"), LLM::default_openai());
    let jerry_handle = env
        .insert_agent(Some("jerry"), agent.clone())
        .await
        .unwrap();

    // Build & insert the Forgetful listener
    let fgt = Forgetful::new("jerry");
    let _ = env.insert_listener(fgt).await;

    // Spawn the environment
    let mut env_handle = env.spawn_handle().unwrap();
    let message = Message::new_user("whats up jerry");

    // We'll get 5 separate completions, each time the agent's memory should be cleared
    for _ in 0..=5 {
        let t = jerry_handle
            .request_io_completion(message.clone())
            .await
            .unwrap();
        // Make sure the completion goes through before requesting another one
        let _ = env_handle.wait_for_notification(&t).await;
    }

    let state_ticket = jerry_handle.request_state().await.unwrap();
    let mut stack = env_handle.finish_current_job().await.unwrap();
    let noti: espionox::environment::dispatch::EnvNotification =
        stack.take_by_ticket(state_ticket).unwrap();

    let jerry_m_stack: &MessageStack = noti.extract_body().try_into().unwrap();
    println!("Jerry stack: {:?}", jerry_m_stack);

    let stack_sans_system = jerry_m_stack.ref_filter_by(MessageRole::System, false);

    // After removing any system prompts, Jerry's cache length should be 0
    assert_eq!(stack_sans_system.len(), 0);
    println!("All asserts passed, forgetful working as expected");
}
