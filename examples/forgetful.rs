use espionox::{
    agents::{
        actions::io_completion,
        error::AgentResult,
        listeners::{AgentListener, ListenerTrigger},
        memory::MessageRole,
        Agent,
    },
    language_models::completions::CompletionModel,
};

/// This is a simple listener that will always ensure a model's memory never has anything more than
/// it's system prompt in it's memory. Useful for internal Summarizer agents
#[derive(Debug)]
pub struct Forgetful {}

impl AgentListener for Forgetful {
    fn trigger<'l>(&self) -> ListenerTrigger {
        ListenerTrigger::from(0)
    }
    fn sync_method<'l>(&'l mut self, agent: &'l mut Agent) -> AgentResult<()> {
        agent.cache.mut_filter_by(MessageRole::System, true);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("OPENAI_KEY").unwrap();

    // Standard boilerplate for building an Environment & Agent
    let mut agent = Agent::new(
        Some("You are jerry!!"),
        CompletionModel::default_openai(&api_key),
    );
    let fgt = Forgetful {};
    agent.insert_listener(fgt);

    for _ in 0..=5 {
        let _ = agent.do_action(io_completion, (), Some(0)).await.unwrap();
    }

    let stack_sans_system = agent.cache.ref_filter_by(MessageRole::System, false);

    // After removing any system prompts, Jerry's cache length should be 0
    assert_eq!(stack_sans_system.len(), 0);
    println!("All asserts passed, forgetful working as expected");
}
