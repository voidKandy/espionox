use crate::handler::agent::Agent;
use crate::handler::operations::Operational;
use inquire::{
    autocompletion::{Autocomplete, Replacement},
    CustomUserError, Text,
};

pub struct Ui<'a> {
    completer: HistoryCompleter,
    agent: &'a mut Agent,
}

#[derive(Default, Debug, Clone)]
pub struct HistoryCompleter {
    input: String,
    history: Vec<String>,
}

impl HistoryCompleter {
    pub fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        Ok(self.input = input.to_owned())
    }
    pub fn update_history(&mut self, str: &str) {
        self.history.push(str.to_string())
    }
}

impl Autocomplete for HistoryCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;
        Ok(self.history.clone())
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
            completer: HistoryCompleter::default(),
            agent,
        }
    }

    pub fn completer(&self) -> impl Autocomplete {
        self.completer.clone()
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
            )
        )
    }

    fn prompt_user(&mut self) -> String {
        let input = Text::new("")
            .with_autocomplete(self.completer())
            .prompt()
            .unwrap();
        self.completer.update_history(&input);
        input.to_string()
    }

    fn handle_user_answer(&mut self, input: &str) {
        match input.chars().nth(0) {
            Some('!') => println!("{}", Agent::run_input(&input[1..])),
            Some('?') => println!(
                "{}",
                self.agent
                    .read_args(input[1..].split_whitespace().collect())
            ),
            Some(_) => println!("{}", self.agent.prompt(&input)),

            // Some(_) => {
            //     let mut rx = agent.stream_prompt(&input);
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
            let ans = self.prompt_user();
            self.handle_user_answer(&ans);
            // agent.context.save_buffer();
        }
    }
}
