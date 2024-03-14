use core::time::Duration;

use espionox::{
    agents::{
        independent::IndependentAgent,
        memory::{messages::MessageRole, Message, MessageStack},
        Agent,
    },
    environment::{
        dispatch::{
            listeners::ListenerMethodReturn, Dispatch, EnvListener, EnvMessage, EnvNotification,
            EnvRequest,
        },
        Environment,
    },
    language_models::endpoint_completions::LLMCompletionHandler,
};

#[derive(Debug)]
pub struct SummarizeAtLimit {
    limit: usize,
    summarizer: IndependentAgent,
    watched_agent_id: String,
}

impl SummarizeAtLimit {
    fn new(limit: usize, watched_agent_id: &str, summarizer: IndependentAgent) -> Self {
        Self {
            limit,
            watched_agent_id: watched_agent_id.to_owned(),
            summarizer,
        }
    }
}

impl EnvListener for SummarizeAtLimit {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        if let EnvMessage::Response(noti) = env_message {
            if let EnvNotification::AgentStateUpdate {
                agent_id, cache, ..
            } = noti
            {
                if cache.len() >= self.limit && agent_id == &self.watched_agent_id {
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
            let cache_to_summarize = match trigger_message {
                EnvMessage::Response(ref noti) => match noti {
                    EnvNotification::AgentStateUpdate { cache, .. } => cache.to_string(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            let message = Message::new_user(&format!(
                "Summarize this chat history: {}",
                cache_to_summarize
            ));
            self.summarizer.agent.cache.push(message);
            let summary = self.summarizer.io_completion().await?;

            let watched_agent = dispatch
                .get_agent_mut(&self.watched_agent_id)
                .expect("Failed to get watched agent");
            watched_agent.cache.mut_filter_by(MessageRole::System, true);
            watched_agent.cache.push(Message::new_system(&summary));
            Ok(trigger_message)
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("TESTING_API_KEY").unwrap();
    let mut env = Environment::new(Some("testing"), Some(&api_key));
    let agent = Agent::default();
    let mut jerry_handle = env.insert_agent(Some("jerry"), agent).await.unwrap();

    let summarizer = env
        .make_agent_independent(Agent {
            cache: MessageStack::new("Your job is to summarize chunks of a conversation"),
            completion_handler: LLMCompletionHandler::default_openai(),
        })
        .await
        .unwrap();
    let sal = SummarizeAtLimit::new(5usize, "jerry", summarizer);

    env.insert_listener(sal).await;
    env.spawn().await.unwrap();
    for _ in 0..=5 {
        jerry_handle
            .request_cache_push(
                "im saying things to fill space".to_owned(),
                MessageRole::System,
            )
            .await
            .expect("failed to request cache push");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    env.finalize_dispatch().await.unwrap();
    let stack = env.notifications.0.write().await;
    let messages = match stack.get(0).unwrap() {
        EnvNotification::AgentStateUpdate { cache, .. } => cache.as_ref(),
        _ => panic!("First on stack should be a cache update"),
    };

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, MessageRole::System);
    assert_eq!(messages[1].role, MessageRole::User);
    println!("All asserts passed, summarize at limit working as expected");
}
