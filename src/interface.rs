use crate::{
    core::file_interface::File,
    handler::{agent::Agent, memorable::Memorable},
};
use colored::*;
use inquire::Text;
use std::process::{Command, Stdio};

fn say_hi() {
    println!(
        "{}",
        format!(
            r#"
                                                          /\
                                                         /  \
                                                        /    \
                                                       /      \
                                                      /        \
                                                     /          \
                                                    /   []  []   \
                                                   /              \
                                            +-----/                \-----+
                                            |    /                  \    |   
                                            |   /                    \   |
                                            |  /                      \  |
                                            | /                        \ |
                                            +----------------------------+
                                            ?                            ?

    "#
        )
        .green()
    )
}

fn run_input(input: &str) -> String {
    let args: Vec<&str> = input.split(" ").collect();
    let out = Command::new(args[0])
        .args(&args[1..])
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute tmux command");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn sop(agent: &mut Agent, input: &str) -> String {
    let args: Vec<&str> = input.split_whitespace().collect();
    match args[0] {
        "rem" => {
            agent.remember(File::build(
                args.get(1).expect("Make sure to pass filepath"),
            ));
            format!("Added {} to buffer.", args.get(1).unwrap())
        }
        "shobuf" => format!("{:?}", agent.context.buffer),
        _ => String::new(),
    }
}

pub fn main_loop() {
    let mut agent = Agent::cache();
    match std::env::var("TMUX") {
        Ok(tmux_var) => println!("📺 Tmux session: {}", tmux_var),
        Err(_) => println!(
            "❗️Make sure your terminal is running inside a Tmux session❗️\n|run src/start.sh|\n"
        ),
    }
    say_hi();
    loop {
        let input = Text::new("").prompt().unwrap();
        let response = match input.chars().nth(0) {
            Some('!') => run_input(&input[1..]).red(),
            Some('?') => sop(&mut agent, &input[1..]).purple(),
            Some(_) => agent.prompt(&input.to_string()).blue(),
            _ => panic!("Something wrong with user input"),
        };
        println!("{response}");
    }
}
