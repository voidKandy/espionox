use consoxide::*;
use tokio;

#[tokio::main]
async fn main() {
    let mut agent = handler::Agent::init();
    interface::Ui::init(&mut agent).interractive_loop();
}
