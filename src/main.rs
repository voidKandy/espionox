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

    // let root = config::Directory::build("test-dir").unwrap();
    // println!("{}", root);
    // let rand_content = &root.files[0].content;
    // println!("{rand_content}");
    let pane = Pane::capture();
    pane.write_to("test-dir/test2.txt").unwrap();
}
