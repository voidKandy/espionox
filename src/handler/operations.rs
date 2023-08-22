use crate::context::memory::Memory;

use super::super::core::{Directory, File};
use super::agent::Agent;
use inquire::{error::InquireError, Confirm, Select, Text};
use std::path::Path;

pub trait Operational {
    fn run_operation(&mut self, args: Vec<&str>) -> String;
    fn switch_memory(&mut self) -> String;
    fn remember_from_path(&mut self, path_str: &str) -> String;
    fn long_term_memory_switch(&mut self);
}

impl Operational for Agent {
    fn run_operation(&mut self, args: Vec<&str>) -> String {
        match args[0] {
            "switch" => self.switch_memory(),
            "rem" => match args.get(1) {
                Some(path) => self.remember_from_path(path),
                None => format!("Please pass a path to the file or directory you would like remembered.\nUsage: 'rem path'")
            }
            "info" => self.context.info_display(),
            "sv" => {
                self.context.save_buffer();
                String::from("Saved Current Message Buffer")
            },
            "help" => format!(r#"
                'switch' - Switch between memory modes
                'rem <file or directory>' - Remember any file or directory
                'info' - get info on the agent's session
                'sv' - save current message buffer
                "#),
            _ => String::from("I don't recognize that command, try ?help"),
        }
    }

    fn switch_memory(&mut self) -> String {
        let options: Vec<&str> = vec!["Long Term", "Short Term", "Forget"];
        let ans: Result<&str, InquireError> = Select::new("Which memory type?", options).prompt();
        match ans {
            Ok("Long Term") => self.long_term_memory_switch(),
            Ok("Short Term") => self.switch_mem(Memory::ShortTerm),
            Ok("Forget") => self.switch_mem(Memory::Forget),
            _ => return String::from("Not a valid argument"),
        };
        format!("Changed memory to {}", ans.unwrap())
    }

    fn long_term_memory_switch(&mut self) {
        let ans = Confirm::new("Create a new memory thread?")
            .with_default(false)
            .prompt();
        let chosen_thread = match ans.unwrap() {
            false => {
                let existing_threads: Vec<String> = self
                    .context
                    .memory
                    .get_active_long_term_threads()
                    .unwrap()
                    .to_vec();
                Select::new("Choose thread to switch to", existing_threads)
                    .prompt()
                    .unwrap()
            }
            true => Text::new("What would you like to name the new thread?")
                .prompt()
                .unwrap(),
        };
        self.switch_mem(Memory::LongTerm(chosen_thread))
    }

    fn remember_from_path(&mut self, path_str: &str) -> String {
        let path = Path::new(path_str);
        if path.is_file() {
            self.remember(File::build(path_str));
        } else if path.is_dir() {
            self.remember(Directory::build(path_str));
        }
        format!("Added {:?} to buffer.", path)
    }
}
