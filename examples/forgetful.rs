use std::collections::HashMap;

use espionox::{
    agents::{memory::Message, Agent},
    environment::{
        agent_handle::{EndpointCompletionHandler, MessageRole},
        dispatch::{listeners::ListenerMethodReturn, Dispatch, EnvListener, EnvMessage},
        Environment,
    },
    language_models::{
        endpoint_completions::LLMCompletionHandler, openai::completions::OpenAiCompletionHandler,
        ModelProvider,
    },
};

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

impl<H: EndpointCompletionHandler> EnvListener<H> for Forgetful {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        match env_message {
            EnvMessage::Response(noti) => {
                if let Some(id) = noti.agent_id() {
                    if id == &self.watched_agent_id {
                        Some(env_message)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch<H>,
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
    let agent = Agent::new(
        "You are jerry!!",
        LLMCompletionHandler::<OpenAiCompletionHandler>::default_openai(),
    );
    let mut jerry_handle = env
        .insert_agent(Some("jerry"), agent.clone())
        .await
        .unwrap();

    let fgt = Forgetful::from("jerry");
    env.insert_listener(fgt).await;
    env.spawn().await.unwrap();
    let message = Message::new_user("whats up jerry");
    for _ in 0..=5 {
        let _ = jerry_handle
            .request_io_completion(message.clone())
            .await
            .unwrap();
    }
    env.finalize_dispatch().await.unwrap();
    let dispatch = env.dispatch.write().await;

    let jerry = dispatch.get_agent_ref(&jerry_handle.id).unwrap();

    println!("Jerry stack: {:?}", jerry.cache);

    assert_eq!(jerry.cache.len(), 0);
    println!("All asserts passed, forgetful working as expected");
}
