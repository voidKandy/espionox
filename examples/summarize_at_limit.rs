use core::time::Duration;
use futures_util::Future;
use std::pin::Pin;

use espionox::{
    environment::{
        agent::memory::{messages::MessageRole, Message, MessageVector},
        dispatch::{Dispatch, EnvListener, EnvMessage, EnvNotification, EnvRequest},
        errors::DispatchError,
        Environment,
    },
    Agent,
};

#[derive(Debug)]
pub struct SummarizeAtLimit {
    limit: usize,
    watched_agent_id: String,
    summarizer_agent_id: String,
}

impl From<(usize, &str, &str)> for SummarizeAtLimit {
    fn from((limit, wa, sa): (usize, &str, &str)) -> Self {
        let watched_agent_id = wa.to_string();
        let summarizer_agent_id = sa.to_string();
        Self {
            limit,
            watched_agent_id,
            summarizer_agent_id,
        }
    }
}

impl EnvListener for SummarizeAtLimit {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        if let EnvMessage::Response(noti) = env_message {
            if let EnvNotification::CacheUpdate { agent_id, cache } = noti {
                if cache.len() >= self.limit && agent_id == &self.watched_agent_id {
                    return Some(env_message);
                }
            }
        }
        None
    }

    fn method<'l>(
        &'l mut self,
        trigger_message: &'l EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> Pin<Box<dyn Future<Output = Result<(), DispatchError>> + Send + Sync + 'l>> {
        Box::pin(async move {
            let client = &dispatch.client.clone();
            let api_key = dispatch.api_key().expect("No api key");
            let cache_to_summarize = match trigger_message {
                EnvMessage::Response(noti) => match noti {
                    EnvNotification::CacheUpdate { cache, .. } => cache.to_string(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };
            let message = Message::new(
                MessageRole::User,
                &format!("Summarize this chat history: {}", cache_to_summarize),
            );
            let summarizer = dispatch
                .get_agent_mut(&self.summarizer_agent_id)
                .expect("Failed to get summarizer");
            tracing::info!("Sending message to summarizer: {:?} ", message);
            let io_comp_fn = summarizer.model.io_completion_fn();
            let mut mvec = MessageVector::init();
            mvec.push(message);

            let summary = io_comp_fn(&client, &api_key, &(&mvec).into(), &summarizer.model)
                .await
                .expect("Failed to get GptResponse");
            let summary = summarizer
                .handle_completion_response(summary)
                .expect("Failed to parse GptResponse");

            let watched_agent = dispatch
                .get_agent_mut(&self.watched_agent_id)
                .expect("Failed to get watched agent");
            watched_agent.cache.reset_to_system_prompt();
            watched_agent
                .cache
                .push(Message::new(MessageRole::System, &summary));
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("TESTING_API_KEY").unwrap();
    let mut env = Environment::new(Some("testing"), Some(&api_key));
    let agent = Agent::default();
    let _ = env
        .insert_agent(Some("jerry"), agent.clone())
        .await
        .unwrap();
    let _ = env.insert_agent(Some("sum"), agent).await.unwrap();

    let sal = SummarizeAtLimit::from((5usize, "jerry", "sum"));
    env.add_listener(sal).await;
    env.spawn().await.unwrap();
    let message = Message::new(MessageRole::User, "im saying things to fill space");
    for _ in 0..=5 {
        let sender = env.clone_sender();
        let sender_lock = sender.lock().await;
        let push_to_cache = EnvRequest::PushToCache {
            agent_id: "jerry".to_string(),
            message: message.clone(),
        };
        sender_lock.send(push_to_cache.into()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    env.finalize_dispatch().await.unwrap();
    let stack = env.notifications.0.write().await;
    let messages = match stack.get(0).unwrap() {
        EnvNotification::CacheUpdate { cache, .. } => cache.as_ref(),
        _ => panic!("First on stack should be a cache update"),
    };

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, MessageRole::System);
    assert_eq!(messages[1].role, MessageRole::User);
    println!("All asserts passed, summarize at limit working as expected");
}
