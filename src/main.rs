pub mod agent;
pub mod session;
pub mod tests;
use agent::agents::{FunctionAgents, PromptAgents};
use agent::gpt::Gpt;
use session::pane::Pane;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    match env::var("TMUX") {
        Ok(tmux_var) => println!("📺 Tmux session: {}", tmux_var),
        Err(_) => println!("❗️Make sure your terminal is running inside a Tmux session❗️"),
    }
    // let gpt = Gpt::init("You are an ai".to_string());

    let special_agent = FunctionAgents::IoAgent;
    let agent = special_agent.init();
    let prompt = special_agent.get_prompt();
    let function = &agent.functions.as_ref().unwrap().get(0);
    let response = agent.fn_prompt(&prompt, &function.unwrap()).await.unwrap();
    let parsed_response = special_agent.parse_response(response);
    println!("{:?}", parsed_response);

    // let root = config::Directory::build("test-dir").unwrap();
    // println!("{}", root);
    // let rand_content = &root.files[0].content;
    // println!("{rand_content}");
    let pane = Pane::capture();
    pane.write_to("test-dir/test2.txt").unwrap();
}
