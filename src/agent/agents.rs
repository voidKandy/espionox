use super::context::config::Context;
use super::context::walk::{Directory, File};
use super::functions::config::Function;
use super::functions::enums::FnEnum;
use super::gpt::Gpt;
use inquire::Text;
use serde_json::{json, Value};
use std::error::Error;

#[derive(Clone)]
pub struct Agent {
    pub handler: Option<Gpt>,
    pub system_prompt: String,
}
#[derive(Clone)]
pub enum SpecialAgent {
    ChatAgent,
    WatcherAgent,
    SummarizeAgent,
}

impl SpecialAgent {
    pub fn init_agent(&self) -> Agent {
        Agent {
            handler: self.get_handler(),
            system_prompt: self.get_sys_prompt(),
        }
    }
    pub fn get_handler(&self) -> Option<Gpt> {
        Some(Gpt::init(self.get_sys_prompt()))
    }
    pub fn get_sys_prompt(&self) -> String {
        match self {
            SpecialAgent::ChatAgent => String::from("You are a state of the art coding ai, help users with any computer programming related questions."),
            SpecialAgent::WatcherAgent => String::from( "You are an Watcher agent, you watch the virtual actions of your human friend and help them."),
            SpecialAgent::SummarizeAgent => String::from("You are a state of the art code summarizing ai. Create a thorough yet succinct summary of the file provided."),
        }
    }
    pub fn get_functions(&self) -> Option<Vec<FnEnum>> {
        match self {
            SpecialAgent::ChatAgent => None,
            SpecialAgent::SummarizeAgent => None,
            SpecialAgent::WatcherAgent => Some(vec![FnEnum::GetCommands, FnEnum::RelevantFiles]),
        }
    }
    pub fn get_user_prompt(&self) -> String {
        match self {
            SpecialAgent::ChatAgent => Text::new("| Chat with me :) | ").prompt().unwrap(),
            SpecialAgent::WatcherAgent => Text::new("| Ayo Whaddup | ").prompt().unwrap(),
            _ => Text::new("|__| ").prompt().unwrap(),
        }
    }
}

impl Agent {
    pub async fn get_completion_response(
        &self,
        context: &Vec<Value>,
    ) -> Result<String, Box<dyn Error>> {
        if let Some(gpt) = &self.handler {
            match gpt.completion(context).await {
                Ok(response) => Ok(response.choices[0].message.content.to_owned().unwrap()),
                Err(err) => Err(err),
            }
        } else {
            Err("Agent doesn't have a handler".into())
        }
    }
    pub async fn get_function_completion_response(
        &self,
        context: &Vec<Value>,
        function: &Function,
    ) -> Result<Value, Box<dyn Error>> {
        if let Some(gpt) = &self.handler {
            match gpt.function_completion(context, function).await {
                Ok(response) => Ok(response
                    .choices
                    .into_iter()
                    .next()
                    .unwrap()
                    .message
                    .function_call
                    .to_owned()
                    .unwrap()),
                Err(err) => Err(err),
            }
        } else {
            Err("Agent doesn't have a handler".into())
        }
    }
}
