use espionox::{agents::error::AgentResult, prelude::*};

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

/// We'll define our own push to cache action method so we can trigger the listener on it
/// All action methods must be async & return AgentResult, which can be coerced from an
/// `anyhow::Result`
async fn push_to_cache_with_limit(
    agent: &mut Agent,
    sum: &mut SummarizeAtLimit,
    m: Message,
) -> AgentResult<()> {
    if agent.cache.len() >= sum.limit {
        let message = Message::new_user(&format!(
            "Summarize this chat history: {}",
            agent.cache.to_string()
        ));
        sum.summarizer.cache.push(message);

        let summary = sum.summarizer.io_completion().await?;

        agent.cache.mut_filter_by(&MessageRole::System, true);
        agent.cache.push(Message::new_assistant(&summary));
    }
    agent.cache.push(m);
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("ANTHROPIC_KEY").unwrap();
    let mut agent = Agent::new(
        Some("You are jerry!!"),
        CompletionModel::default_anthropic(&api_key),
    );

    let summarizer = Agent::new(
        Some("Your job is to summarize chunks of a conversation"),
        CompletionModel::default_anthropic(&api_key),
    );
    let mut sal = SummarizeAtLimit::new(5usize, summarizer);

    // agent.insert_listener(sal);
    let message = Message::new_user("im saying things to fill space");

    for _ in 0..=5 {
        // And now we use our predefined action method
        push_to_cache_with_limit(&mut agent, &mut sal, message.clone())
            .await
            .unwrap();
    }

    // env.finalize_dispatch().await.unwrap();
    println!("STACK: {:?}", agent.cache);
    assert_eq!(agent.cache.len(), 4);
    assert_eq!(agent.cache.as_ref()[0].role, MessageRole::System);
    assert_eq!(agent.cache.as_ref()[1].role, MessageRole::Assistant);
    assert_eq!(agent.cache.as_ref()[2].role, MessageRole::User);
    println!("All asserts passed, summarize at limit working as expected");
}
