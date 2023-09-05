use consoxide::{
    agent::{Agent, AgentSettings},
    interface::terminal::Ui,
};
use tokio;

#[tokio::main]
async fn main() {
    let mut agent = Agent::build(AgentSettings::default()).expect("Failed to build agent");
    Ui::init(Some(&mut agent)).interractive_loop();
}
