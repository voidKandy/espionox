use consoxide::{handler::AgentSettings, *};
use tokio;

#[tokio::main]
async fn main() {
    let mut agent = handler::Agent::build(AgentSettings::default()).expect("Failed to build agent");
    interface::Ui::init(&mut agent).interractive_loop();
}
