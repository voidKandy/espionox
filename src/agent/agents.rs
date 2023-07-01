use super::fn_enums::FnEnum;
use super::fn_render::Function;
use super::gpt::Gpt;
use inquire::Text;
use serde_json::Value;
use std::error::Error;
use std::fs;

pub struct Agent {
    pub handler: Option<Gpt>,
    pub system_prompt: String,
    pub functions: Option<Vec<Function>>,
}
pub enum PromptAgents {
    ChatAgent,
    // BrainstormAgent(Prompt),
}
pub enum FunctionAgents {
    IoAgent,
}
//
impl Agent {
    pub async fn prompt(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        if let Some(gpt) = &self.handler {
            match gpt.completion(&prompt).await {
                Ok(response) => Ok(response.choices[0].message.content.to_owned().unwrap()),
                Err(err) => Err(err),
            }
        } else {
            Err("Agent doesn't have a handler".into())
        }
    }
    pub async fn fn_prompt(
        &self,
        prompt: &str,
        function: &Function,
    ) -> Result<Value, Box<dyn Error>> {
        if let Some(gpt) = &self.handler {
            match gpt.function_completion(prompt, function).await {
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

impl PromptAgents {
    pub fn init(&self) -> Agent {
        Agent {
            handler: self.get_handler(),
            system_prompt: self.get_sys_prompt(),
            functions: None,
        }
    }
    pub fn get_prompt(&self) -> String {
        Text::new("Ayo whaddup").prompt().unwrap()
    }
    pub fn get_handler(&self) -> Option<Gpt> {
        Some(Gpt::init(self.get_sys_prompt()))
    }
    pub fn get_sys_prompt(&self) -> String {
        match self {
            PromptAgents::ChatAgent => String::from("You are a state of the art coding ai, help users with any computer programming related questions."),
        }
    }
}

impl FunctionAgents {
    pub fn init(&self) -> Agent {
        Agent {
            handler: self.get_handler(),
            system_prompt: self.get_sys_prompt(),
            functions: self.get_functions(),
        }
    }
    pub fn get_prompt(&self) -> String {
        Text::new("Here to do some operations ⚙️").prompt().unwrap()
    }
    pub fn get_handler(&self) -> Option<Gpt> {
        Some(Gpt::init(self.get_sys_prompt()))
    }
    pub fn get_functions(&self) -> Option<Vec<Function>> {
        match self {
            FunctionAgents::IoAgent => Some(vec![FnEnum::GetCommands.get_function()]),
        }
    }
    pub fn get_sys_prompt(&self) -> String {
        match self {
            FunctionAgents::IoAgent => String::from(
                "You are an Io agent, you perform simple IO functions based on user input",
            ),
        }
    }
    pub fn parse_response(&self, response: Value) -> Option<Vec<String>> {
        let arguments = response.get("arguments")?.as_str()?;
        let parsed_arguments = serde_json::from_str::<Value>(arguments).ok()?;
        let commands = parsed_arguments.get("commands")?.as_array()?;
        let command_strings = commands
            .iter()
            .filter_map(|command| command.as_str().map(String::from))
            .collect::<Vec<String>>();
        Some(command_strings)
    }
}
// impl BrainstormAgent {
//     pub fn init() -> BrainstormAgent {
//         let sys_message = "You are a brainstorming ai, you take requests from users and output the most detailed possible step by step plan to execute their needs.".to_string();
//         BrainstormAgent(Agent::init)
//     }
// }
//
// ------------------- Function Agents ------------------- //
// pub struct IoAgent(pub Agent);

// impl IoAgent {
//     pub fn init() -> IoAgent {
//         let sys_message =
//             "You are an Io agent, you perform simple IO functions based on user input".to_string();
//         IoAgent(Agent::init(Gpt::init(sys_message)))
//     }
// pub async fn get_commands(&self, prompt: &str) -> Vec<String> {
//     let function = FnEnum::GetCommands.get_function();
//     self.0.fn_prompt(prompt, function).await
// }
// }
// impl PromptAgent {
//     pub async fn prompt(&self, prompt: &str) -> Result<GptResponse, Box<dyn Error>> {
//         // Prompt agent implementation specific to each variant
//         match self {
//             PromptAgent::ChatAgent(agent) => agent.0.prompt(prompt).await,
//             PromptAgent::BrainstormAgent(agent) => agent.0.prompt(prompt).await,
//         }
//     }
//
//     pub async fn summarize_file(&self, filepath: &str) -> String {
//         let file_contents = fs::read_to_string(filepath).expect("Couldn't read that boi");
//         let prompt = format!("Summarize the contents of this file: {}", file_contents);
//
//         let response = self.prompt(&prompt).await.unwrap();
//         response.choices[0].message.content.to_owned()
//     }
// }
// impl FunctionAgent {
//     pub async fn fn_prompt(
//         &self,
//         prompt: &str,
//         function: Function,
//     ) -> Result<GptResponse, Box<dyn Error>> {
//         // Function agent implementation specific to each variant
//         match self {
//             FunctionAgent::IoAgent => self.0.function_prompt(prompt, function).await,
//         }
//     }
// }
