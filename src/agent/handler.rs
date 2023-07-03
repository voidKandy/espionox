use super::context::manager::Context;
use super::context::walk::File;
use super::functions::config::Function;
use super::functions::enums::FnEnum;
use super::gpt::Gpt;
use inquire::Text;
use serde_json::Value;
use std::error::Error;

pub struct AgentHandler {
    pub special_agent: SpecialAgent,
    pub agent: Agent,
    pub context: Vec<Value>,
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
    SummarizeAgent,
}

impl AgentHandler {
    pub fn new(special_agent: SpecialAgent) -> AgentHandler {
        AgentHandler {
            special_agent: special_agent.clone(),
            agent: special_agent.init_agent(),
            context: special_agent.init_context(None),
        }
    }
    pub async fn prompt(&self) -> Result<String, Box<dyn Error>> {
        self.agent.get_completion_response(&self.context).await
    }
    pub async fn function_prompt(&self, function: &Function) -> Result<Value, Box<dyn Error>> {
        self.agent
            .get_function_completion_response(&self.context, &function)
            .await
    }
    pub fn update_context(&mut self, role: &str, content: &str) -> Result<(), Box<dyn Error>> {
        self.special_agent
            .append_message(&mut self.context, role, content);
        Ok(())
    }
    pub async fn summarize_file(&mut self, file: File) -> Result<String, Box<dyn Error>> {
        match self.special_agent {
            SpecialAgent::SummarizeAgent => {
                self.special_agent.append_files(
                    &mut self.context,
                    vec![file],
                    "Here is the file: ",
                );
                self.prompt().await
            }
            _ => Err("Summarize only implemented for summarize agent".into()),
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
            SpecialAgent::SummarizeAgent => String::from("You are a state of the art code summarizing ai. Create a thorough yet succinct summary of the file provided."),
        }
    }
    pub fn get_functions(&self) -> Option<Vec<FnEnum>> {
        match self {
            SpecialAgent::ChatAgent => None,
            SpecialAgent::SummarizeAgent => None,
            SpecialAgent::IoAgent => Some(vec![FnEnum::GetCommands, FnEnum::RelevantFiles]),
        }
    }
    pub fn get_user_prompt(&self) -> String {
        match self {
            SpecialAgent::ChatAgent => Text::new("Chat with me :)").prompt().unwrap(),
            SpecialAgent::IoAgent => Text::new("Here to do some operations ⚙️").prompt().unwrap(),
            _ => Text::new("Ayo whaddup").prompt().unwrap(),
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
