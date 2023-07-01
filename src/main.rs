pub mod agent;
pub mod session;
pub mod tests;
use agent::agents::Agent;
use agent::fn_enums::FnEnum;
use session::pane::Pane;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    match env::var("TMUX") {
        Ok(tmux_var) => println!("📺 Tmux session: {}", tmux_var),
        Err(_) => println!("❗️Make sure your terminal is running inside a Tmux session❗️"),
    }
    let agent = Agent::ChatAgent::init();
    let prompt = agent.initial_prompt();
    let function = FnEnum::GetCommands.get_function();
    let response = agent.prompt(&prompt).await;
    println!("{:?}", response);
    // let root = config::Directory::build("test-dir").unwrap();
    // println!("{}", root);
    // let rand_content = &root.files[0].content;
    // println!("{rand_content}");
    let pane = Pane::capture();
    pane.write_to("test-dir/test2.txt").unwrap();
}
