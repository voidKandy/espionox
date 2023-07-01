use super::fn_render::Function;
use super::gpt::Gpt;
use inquire::Text;
use std::error::Error;
use std::fs;
// pub Gpt

pub struct GptBody {
    pub handler: Option<Gpt>,
    pub system_prompt: String,
    pub functions: Option<Vec<Function>>,
}
// pub enum Agents {
//     ChatAgent,
//     BrainstormAgent,
//     IoAgent,
// }
//
impl GptBody {
    pub fn create_handler(&mut self) {
        self.handler = Some(Gpt::init(self.system_prompt))
    }
    pub async fn prompt(
        &self,
        prompt: &str,
        function: Option<Function>,
    ) -> Result<String, Box<dyn Error>> {
        if let Some(gpt) = &self.handler {
            match gpt.completion(&prompt).await {
                Ok(response) => Ok(response.choices[0].message.content.to_owned()),
                Err(err) => Err(err),
            }
        } else {
            Err("No handler in GptBody".into())
        }
    }
    pub async fn fn_prompt(
        &self,
        prompt: &str,
        function: Function,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        if let Some(gpt) = &self.handler {
            match gpt.function_completion(prompt, function).await {
                Ok(response) => Ok(response
                    .choices
                    .into_iter()
                    .map(|c| c.message.content)
                    .collect()),
                Err(err) => Err(err),
            }
        } else {
            Err("No handler in GptBody".into())
        }
    }
}

pub trait Agent {
    fn init(&self) -> GptBody;
    fn initial_prompt(&self) -> String {
        Text::new("Ayo whaddup").prompt().unwrap()
    }
}

// ------------------- Prompt Agents ------------------- //
// pub struct BrainstormAgent(pub Agent);
pub struct Chat {
    pub functions: Vec<Function>,
}

impl Agent for Chat {
    fn init(&self) -> GptBody {
        let mut gpt = GptBody {
            handler: None,
            system_prompt: String::from("You are a super cool chatbot ai 🪃"),
            functions: Some(self.functions),
        };
        gpt.create_handler();
        gpt
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
