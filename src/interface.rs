use crate::core::Io;
use crate::handler::agent::Agent;
use crate::handler::operations::Operational;
use colored::*;
use inquire::{
    autocompletion::{Autocomplete, Replacement},
    CustomUserError, Text,
};

pub struct Ui<'a> {
    // completer: CommandCompleter,
    agent: &'a mut Agent,
}

#[derive(Default, Debug, Clone)]
pub struct CommandCompleter {
    input: String,
    commands: Vec<String>,
}

impl CommandCompleter {
    pub fn init() -> Self {
        let commands = vec![
            "?switch".to_string(),
            "?rem".to_string(),
            "?info".to_string(),
        ];
        CommandCompleter {
            input: "".to_string(),
            commands,
        }
    }
    pub fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        Ok(self.input = input.to_owned())
    }
}

impl Autocomplete for CommandCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;
        Ok(self.commands.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;
        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => Replacement::None,
        })
    }
}
impl<'a> Ui<'a> {
    pub fn init(agent: &'a mut Agent) -> Self {
        Ui {
            // completer: CommandCompleter::init(),
            agent,
        }
    }

    fn handle_user_answer(&mut self, ans: &str) {
        match ans.chars().nth(0) {
            Some('!') => {
                let io = Io::new(&ans[1..]);
                println!("{}", io.o.red());
                self.agent.remember(io);
            }
            Some('?') => {
                let args: Vec<&str> = ans[1..].split_whitespace().collect();
                println!("{}", self.agent.run_operation(args));
            }
            Some(_) => println!("{}", self.agent.prompt(&ans)),

            // Some(_) => {
            //     let mut rx = agent.stream_prompt(&ans);
            //     tokio::spawn(async move {
            //         while let Some(Ok(output)) = rx.recv().await {
            //             print!("{}", output);
            //             std::thread::sleep(std::time::Duration::from_millis(200));
            //         }
            // });
            // }
            _ => println!("Didn't quite get that"),
        };
    }

    pub fn interractive_loop(&mut self) {
        Self::greet();
        loop {
            let ans = Text::new("")
                .with_help_message("? for agent commands, ! for shell commands")
                .prompt()
                .unwrap();
            self.handle_user_answer(&ans);
        }
    }

    fn greet() {
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
            ).magenta()
        )
    }
}
