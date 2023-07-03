use super::functions::enums::FnEnum;
use super::gpt::Gpt;
use inquire::Text;

#[derive(Clone)]
pub enum SpecialAgent {
    ChatAgent,
    WatcherAgent,
    SummarizeAgent,
}

impl SpecialAgent {
    pub fn get_gpt(&self) -> Gpt {
        Gpt::init(self.get_sys_prompt())
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
