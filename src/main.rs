pub mod agent;
pub mod session;
pub mod tests;
use agent::functions::FnEnum;
use session::pane::Pane;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    match env::var("TMUX") {
        Ok(tmux_var) => println!("Current terminal is running in Tmux session: {}", tmux_var),
        Err(_) => println!("Make sure your terminal is running inside a Tmux session."),
    }
    let agent = agent::Agent::init();
    let function = FnEnum::GetCommands.get_function();
    println!("{}", function.render());
    let prompt = agent.initial_prompt();
    let _ = agent.function_prompt(&prompt, function).await;
    // println!(
    //     "{}",
    //     agent.prompt(&prompt).await.unwrap().choices[0]
    //         .message
    //         .content
    // );
    // let root = config::Directory::build("test-dir").unwrap();
    // println!("{}", root);
    // let rand_content = &root.files[0].content;
    // println!("{rand_content}");
    let pane = Pane::capture();
    pane.write_to("test-dir/test2.txt").unwrap();
}
