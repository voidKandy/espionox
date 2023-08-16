use crate::handler::agent::Agent;
use crate::handler::operations::Operational;
use inquire::Text;

pub struct Ui<'a> {
    completer: HistoryCompleter,
    agent: &'a mut Agent,
}

#[derive(Default, Debug)]
pub struct HistoryCompleter {
    input: String,
    history: Vec<String>,
}

impl HistoryCompleter {
    pub fn update(&mut self, str: &str) {
        self.history.push(str.to_string())
    }
}

impl<'a> Ui<'a> {
    pub fn init(agent: &'a mut Agent) -> Self {
        Ui {
            completer: HistoryCompleter::default(),
            agent,
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
            )
        )
    }

    fn prompt_user(&mut self) -> String {
        let input = Text::new("").prompt().unwrap();
        self.completer.update(&input);
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
