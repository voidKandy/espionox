pub mod agent;
pub mod tests;
use std::env;
use tokio;

#[tokio::main]
async fn main() {
    match env::var("TMUX") {
        Ok(tmux_var) => println!("📺 Tmux session: {}", tmux_var),
        Err(_) => println!(
            "❗️Make sure your terminal is running inside a Tmux session❗️\n|run src/start.sh|\n"
        ),
    }
}
