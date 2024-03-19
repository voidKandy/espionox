use std::{collections::HashMap, time::Duration};

use espionox::{
    agents::{
        memory::{Message, MessageStack},
        Agent,
    },
    environment::{
        agent_handle::MessageRole,
        dispatch::{
            listeners::ListenerMethodReturn, Dispatch, EnvListener, EnvMessage, EnvRequest,
        },
        Environment,
    },
    language_models::{ModelProvider, LLM},
};
use tokio::time::sleep;

#[derive(Debug)]
pub struct Forgetful {
    watched_agent_id: String,
}

impl From<&str> for Forgetful {
    fn from(wa: &str) -> Self {
        let watched_agent_id = wa.to_string();
        Self { watched_agent_id }
    }
}

impl EnvListener for Forgetful {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        if let EnvMessage::Request(req) = env_message {
            if let EnvRequest::GetCompletion { agent_id, .. } = req {
                if agent_id == &self.watched_agent_id {
                    return Some(env_message);
                }
            }
        }
        None
    }

    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            let watched_agent = dispatch
                .get_agent_mut(&self.watched_agent_id)
                .expect("Failed to get watched agent");
            watched_agent.cache.mut_filter_by(MessageRole::System, true);
            Ok(trigger_message)
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("OPENAI_KEY").unwrap();
    let mut map = HashMap::new();
    map.insert(ModelProvider::OpenAi, api_key);
    let mut env = Environment::new(Some("testing"), map);
    let agent = Agent::new(Some("You are jerry!!"), LLM::default_openai());
    let mut jerry_handle = env
        .insert_agent(Some("jerry"), agent.clone())
        .await
        .unwrap();

    let fgt = Forgetful::from("jerry");
    let _ = env.insert_listener(fgt).await;
    let mut env_handle = env.spawn_handle().unwrap();
    let message = Message::new_user("whats up jerry");
    for _ in 0..=5 {
        let _ = jerry_handle
            .request_io_completion(message.clone())
            .await
            .unwrap();
        sleep(Duration::from_millis(200)).await;
    }
    let state_ticket = jerry_handle.request_state().await.unwrap();
    let mut stack = env_handle.finish_current_job().await.unwrap();
    let noti: espionox::environment::dispatch::EnvNotification =
        stack.take_by_ticket(state_ticket).unwrap();

    let jerry_m_stack: &MessageStack = noti.extract_body().try_into().unwrap();

    println!("Jerry stack: {:?}", jerry_m_stack);
    let stack_sans_system = jerry_m_stack.ref_filter_by(MessageRole::System, false);
    assert_eq!(stack_sans_system.len(), 0);
    println!("All asserts passed, forgetful working as expected");
}
