use super::fn_enums::FnEnum;
use super::fn_render::Function;
use super::gpt::Gpt;
use inquire::Text;
use serde_json::Value;
use std::error::Error;
use std::fs;

pub struct AgentHandler {
    pub special_agent: SpecialAgent,
    pub agent: Agent,
}

#[derive(Clone)]
pub struct Agent {
    pub handler: Option<Gpt>,
    pub system_prompt: String,
}
#[derive(Clone)]
pub enum SpecialAgent {
    ChatAgent,
    IoAgent,
}

// struct for handling two agents
impl AgentHandler {
    pub fn new(agent: SpecialAgent) -> AgentHandler {
        AgentHandler {
            special_agent: agent.clone(),
            agent: agent.init_agent(),
        }
    }
    pub fn get_prompt(&self) -> String {
        match self.special_agent {
            SpecialAgent::ChatAgent => Text::new("Ayo whaddup").prompt().unwrap(),
            SpecialAgent::IoAgent => Text::new("Here to do some operations ⚙️").prompt().unwrap(),
        }
    }
}

// Struct for instantiating Agent
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
            SpecialAgent::IoAgent => String::from(
                "You are an Io agent, you perform simple IO functions based on user input",
            ),
        }
    }
    pub fn get_functions(&self) -> Option<Vec<FnEnum>> {
        match self {
            SpecialAgent::ChatAgent => None,
            SpecialAgent::IoAgent => Some(vec![FnEnum::GetCommands, FnEnum::RelevantFiles]),
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

// Async Struct
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
    pub async fn function_prompt(
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
    pub async fn summarize_file(&self, filepath: &str) -> String {
        let file_contents = fs::read_to_string(filepath).expect("Couldn't read that boi");
        let prompt = format!("Summarize the contents of this file: {}", file_contents);

        let response = self.prompt(&prompt).await.unwrap();
        response.to_owned()
    }
}
