use core::time::Duration;
use std::collections::HashMap;

use espionox::{
    agents::{
        actions::io_completion,
        error::AgentResult,
        listeners::{AgentListener, ListenerTrigger},
        memory::{messages::MessageRole, Message},
        Agent,
    },
    language_models::{ModelProvider, LLM},
};

#[derive(Debug)]
pub struct SummarizeAtLimit {
    limit: usize,
    summarizer: Agent,
}

impl SummarizeAtLimit {
    fn new(limit: usize, summarizer: Agent) -> Self {
        Self { limit, summarizer }
    }
}

impl AgentListener for SummarizeAtLimit {
    fn trigger<'l>(&self) -> espionox::agents::listeners::ListenerTrigger {
        "sum".into()
    }

    fn async_method<'l>(
        &'l mut self,
        _a: &'l mut Agent,
    ) -> espionox::agents::listeners::ListenerCallReturn<'l> {
        Box::pin(async move {
            let message = Message::new_user(&format!(
                "Summarize this chat history: {}",
                _a.cache.to_string()
            ));
            self.summarizer.cache.push(message);

            let summary = self
                .summarizer
                .do_action(io_completion, (), Option::<ListenerTrigger>::None)
                .await?;

            _a.cache.mut_filter_by(MessageRole::System, true);
            _a.cache.push(Message::new_assistant(&summary));
            Ok(())
        })
    }
}

/// We'll define our own push to cache action method so we can trigger the listener on it
/// All action methods must be async
async fn push_to_cache(agent: &mut Agent, m: Message) -> AgentResult<()> {
    agent.cache.push(m);
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_KEY").unwrap();
    let mut agent = Agent::new(Some("You are jerry!!"), LLM::default_anthropic(&api_key));

    let summarizer = Agent::new(
        Some("Your job is to summarize chunks of a conversation"),
        LLM::default_anthropic(&api_key),
    );
    let sal = SummarizeAtLimit::new(5usize, summarizer);

    agent.insert_listener(sal);
    let message = Message::new_user("im saying things to fill space");

    for _ in 0..=5 {
        // And now we use our predefined action method
        agent
            .do_action(push_to_cache, message.clone(), Some("sum"))
            .await
            .unwrap();
    }

    // env.finalize_dispatch().await.unwrap();
    println!("STACK: {:?}", agent.cache);
    assert_eq!(agent.cache.len(), 3);
    assert_eq!(agent.cache.as_ref()[0].role, MessageRole::System);
    assert_eq!(agent.cache.as_ref()[1].role, MessageRole::Assistant);
    assert_eq!(agent.cache.as_ref()[2].role, MessageRole::User);
    println!("All asserts passed, summarize at limit working as expected");
}
