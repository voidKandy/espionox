use consoxide::{
    agent::{Agent, AgentSettings},
    interfaces::terminal::Ui,
};
use tokio;

#[tokio::main]
async fn main() {
    let mut agent = Agent::default();
    Ui::init(Some(&mut agent)).interractive_loop();
}
