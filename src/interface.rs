use crate::context::Memory;
#[allow(unused)]
use crate::core::{Directory, File, Io};
use crate::handler::{agent::Agent, integrations::*};
use colored::*;
use inquire::{Confirm, InquireError, Select, Text};
use std::path::Path;

pub struct Ui<'a> {
    agent: Option<&'a mut Agent>,
    responder: Option<UiResponder>,
}

type AgentCommands = Vec<String>;

pub trait UiResponse<'a> {}
impl<'a> UiResponse<'a> for AgentCommands {}
impl<'a> UiResponse<'a> for Io {}
impl<'a> UiResponse<'a> for String {}

#[derive(Copy, Clone)]
enum Op {
    SwitchMem,
    Format,
    Info,
    Save,
    Help,
}

impl TryFrom<AgentCommands> for Op {
    type Error = Box<dyn std::error::Error>;
    fn try_from(c: AgentCommands) -> Result<Self, Self::Error> {
        for op_variant in &[Op::SwitchMem, Op::Format, Op::Info, Op::Save, Op::Help] {
            if c[0].as_str() == op_variant.command() {
                return Ok(*op_variant);
            }
        }
        Err("No valid command passed".into())
    }
}

impl Op {
    fn command(&self) -> &str {
        match self {
            Op::SwitchMem => "switch",
            Op::Format => "fmt",
            Op::Info => "info",
            Op::Save => "sv",
            Op::Help => "help",
        }
    }

    fn description(&self) -> &str {
        match self {
            Op::SwitchMem => "Switch between memory modes",
            Op::Format => "Format a file or directory to the current buffer",
            Op::Info => "Display diagnostics on the Ui's current agent",
            Op::Save => "Save the current buffer to the current loaded agent's memory",
            Op::Help => "Display help",
        }
    }

    fn help_message() -> String {
        let mut help = String::new();
        for op_variant in &[Op::SwitchMem, Op::Format, Op::Info, Op::Save, Op::Help] {
            help.push_str(&format!(
                "{} - {}\n",
                op_variant.command(),
                op_variant.description()
            ));
        }
        help
    }
}

enum UiResponder {
    ShellCommand(Io),
    AgentOp(AgentCommands),
    Converse(String),
}

impl From<&str> for UiResponder {
    fn from(s: &str) -> Self {
        match s.chars().nth(0).unwrap() {
            '!' => Self::ShellCommand(Io::new(&s[1..])),
            '?' => Self::AgentOp(
                s[1..]
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<AgentCommands>(),
            ),
            _ => Self::Converse(s.to_string()),
        }
    }
}

impl<'a> Ui<'a> {
    pub fn init(agent: Option<&'a mut Agent>) -> Self {
        Ui {
            agent,
            responder: None,
        }
    }

    pub fn interractive_loop(&mut self) {
        Self::greet();
        loop {
            let ans = Text::new("")
                .with_help_message("? for agent commands, ! for shell commands")
                .prompt()
                .unwrap();
            self.update_responder(&ans);
            self.respond();
        }
    }

    fn update_responder(&mut self, s: &str) {
        self.responder = Some(s.into());
    }

    fn respond(&mut self) {
        if let Some(agent) = self.agent.as_mut() {
            if let Some(res) = &self.responder {
                match res {
                    UiResponder::Converse(input) => {
                        let res = agent.prompt(&input);
                        println!("{}", res.green());
                    }
                    UiResponder::ShellCommand(io) => {
                        println!("{}", io.o.red());
                        agent.format_to_buffer(io.to_owned());
                    }
                    UiResponder::AgentOp(commands) => {
                        match Op::try_from(commands.to_owned() as AgentCommands) {
                            Ok(op) => {
                                match self.execute_op(op) {
                                    Some(message) => println!("{}", message),
                                    None => println!("No message returned from execution"),
                                };
                            }
                            Err(err) => println!(
                                "Agent operator could not be built: {:?}\nTry '?help'",
                                err
                            ),
                        }
                    }
                }
            }
        }
    }

    fn execute_op(&mut self, op: Op) -> Option<String> {
        if let Some(agent) = self.agent.as_mut() {
            if let Some(res) = &self.responder.take() {
                if let UiResponder::AgentOp(command) = res {
                    let response = match op {
                        Op::SwitchMem => Some(self.switch_agent_memory()),
                        Op::Format => match command.get(1) {
                            Some(path) => Some(self.remember_from_path(path)),
                            None => {
                                Some(format!("Please path a valid path to a file or directory"))
                            }
                        },
                        Op::Info => Some(agent.info_display_string()),
                        Op::Save => {
                            agent.context.save_buffer();
                            Some(String::from("Saved current message buffer"))
                        }
                        Op::Help => Some(Op::help_message()),
                    };
                    return response;
                }
            }
            return Some("No responder in UI".to_string());
        }
        Some("No agent in UI".to_string())
    }

    fn switch_agent_memory(&mut self) -> String {
        if let Some(agent) = self.agent.as_mut() {
            let options: Vec<&str> = vec!["Long Term", "Short Term", "Forget"];
            let ans: Result<&str, InquireError> =
                Select::new("Which memory type?", options).prompt();
            match ans {
                Ok("Long Term") => self.long_term_memory_switch(),
                Ok("Short Term") => agent.switch_mem(Memory::ShortTerm),
                Ok("Forget") => agent.switch_mem(Memory::Forget),
                _ => return String::from("Not a valid argument"),
            };
            return format!("Changed memory to {}", ans.unwrap());
        }
        String::from("No agent in UI to change memory")
    }

    fn long_term_memory_switch(&mut self) {
        if let Some(agent) = self.agent.as_mut() {
            let ans = Confirm::new("Create a new memory thread?")
                .with_default(false)
                .prompt();
            let chosen_thread = match ans.unwrap() {
                false => {
                    let existing_threads: Vec<String> = agent
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
            agent.switch_mem(Memory::LongTerm(chosen_thread))
        }
    }

    fn remember_from_path(&mut self, path_str: &str) -> String {
        if let Some(agent) = self.agent.as_mut() {
            let path = Path::new(path_str);
            if path.is_file() {
                agent.format_to_buffer(File::from(path_str));
            } else if path.is_dir() {
                agent.format_to_buffer(Directory::from(path_str));
            }
            return format!("Added {:?} to buffer.", path);
        }
        String::from("No agent to add buffer to")
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
