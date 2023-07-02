pub mod agent;
pub mod session;
pub mod tests;
use agent::agents::*;
use session::pane::Pane;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    match env::var("TMUX") {
        Ok(tmux_var) => println!("📺 Tmux session: {}", tmux_var),
        Err(_) => println!("❗️Make sure your terminal is running inside a Tmux session❗️"),
    }
    let handler = AgentHandler::new(SpecialAgent::IoAgent);

    let prompt = handler.get_prompt();
    // Need to improve function calling
    let function = &handler
        .1
        .functions
        .as_ref()
        .unwrap()
        .get(1)
        .unwrap()
        .to_function();
    let response = handler.1.fn_prompt(&prompt, &function).await.unwrap();
    let parsed_response = handler.0.parse_response(response);
    println!("{:?}", parsed_response);

    // let root = config::Directory::build("test-dir").unwrap();
    // println!("{}", root);
    // let rand_content = &root.files[0].content;
    // println!("{rand_content}");
    let pane = Pane::capture();
    pane.write_to("test-dir/test2.txt").unwrap();
}
