use crate::handler::agent::Agent;
use crate::handler::operations::Operational;
use colored::*;
use inquire::Text;

fn say_hi() {
    println!(
        "{}",
        format!(
            r#"









                                                                                --- [ Hello Human ] ---


                                            
                                                                            
                                                                                          /\
                                                                                         /  \
                                                                                        /    \
                                                                                       /      \
                                                                                      /        \
                                                                                     /          \
                                                                                    /            \
                                                                                   /              \
                                                                            +     /                \     +
                                                                            |    /                  \    |   
                                                                            |   /                    \   |
                                                                            |  /                      \  |
                                                                            | /  ...._          _....  \ |
                                                                            |/  / __  | ______ |  __ \  \|
                                                                            ++--|____/          \____|--++
                                                                             |                          |
                                                                             |                          \   
                                                                             /                           \      
                                                                             / /  |                      /        
                                                                             \/\  /            /   \    /
                                                                                \/\ |   /|    /\   |\  /
                                                                                   \|  /  \  /  \  // /             
                                                                                     \/   | /    \/ \/
                                                                                          \/






    "#
        )
        .green()
    )
}

pub fn main_loop() {
    let mut agent = Agent::init();
    match std::env::var("TMUX") {
        Ok(tmux_var) => println!("📺 Tmux session: {}", tmux_var),
        Err(_) => println!(
            "❗️Make sure your terminal is running inside a Tmux session❗️\n|run src/start.sh|\n"
        ),
    }
    say_hi();
    loop {
        let input = Text::new("").prompt().unwrap();
        match input.chars().nth(0) {
            Some('!') => println!("{}", Agent::run_input(&input[1..]).red()),
            Some('?') => println!(
                "{}",
                agent
                    .read_args(input[1..].split_whitespace().collect())
                    .purple()
            ),
            Some(_) => println!("{}", agent.prompt(&input).magenta()),

            // Some(_) => {
            //     let mut rx = agent.stream_prompt(&input);
            //     tokio::spawn(async move {
            //         while let Some(Ok(output)) = rx.recv().await {
            //             print!("{}", output.magenta());
            //             std::thread::sleep(std::time::Duration::from_millis(200));
            //         }
            // });
            // }
            _ => println!("Didn't quite get that"),
        };
        // agent.context.save_buffer();
    }
}
