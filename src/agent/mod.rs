pub mod functions;
use inquire::Text;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::json;
use std::env;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct GptResponse {
    pub choices: Vec<Choice>,
}

// todo!("create Gpt Response enum for both chat & function ");
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub content: String,
}

pub struct Agent {
    pub config: AgentConfig,
    pub permissions: AgentPermissions,
}

pub struct AgentPermissions {
    pub write: bool,
    pub read: bool,
    pub execute: bool,
}

pub struct AgentConfig {
    pub api_key: String,
    pub client: Client,
    pub url: String,
    pub system_message: String,
}

impl AgentConfig {
    pub fn init(system_message: String) -> AgentConfig {
        dotenv::dotenv().ok();
        let api_key = env::var("OPEN_AI_API_KEY").unwrap();
        let client = Client::new();
        let url = "https://api.openai.com/v1/chat/completions".to_string();
        AgentConfig {
            api_key,
            client,
            url,
            system_message,
        }
    }
}

impl Agent {
    pub fn init() -> Agent {
        let config = AgentConfig::init("You are an ai that summarizes code. Be as thorough as possible while also being as succinct as possible".to_string());
        let permissions = AgentPermissions {
            write: true,
            read: true,
            execute: true,
        };
        Agent {
            config,
            permissions,
        }
    }
    pub fn initial_prompt(&self) -> String {
        Text::new("Ayo whaddup").prompt().unwrap()
    }
    pub async fn prompt(
        &self,
        prompt: &str,
    ) -> Result<GptResponse, Box<dyn std::error::Error>> {
        let response = self.config.client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&json!({ "model": "gpt-3.5-turbo", "messages": [{"role": "system", "content": self.config.system_message}, {"role": "user", "content": prompt}], "max_tokens": 2000, "n": 1, "stop": null }))
            .send()
            .await?;

        let gpt_response: GptResponse = response.json().await?;
        Ok(gpt_response)
    }
    pub async fn function_prompt(
        &self,
        prompt: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("{:?}", format!("bearer {}", self.config.api_key));
        let response = self.config.client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("content-type", "application/json")
            .json(&json!({ 
                "model": "gpt-3.5-turbo-0613",
                "messages": [{"role": "system", "content": self.config.system_message}, {"role": "user", "content": prompt}], 
                "functions": [
                {
                  "name": "get_commands",
                  "description": "get a list of terminal commands to run on mac os",
                  "parameters": {
                    "type": "object",
                    "properties": {
                      "commands": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "description": "a terminal command string"
                        },
                        "description": "list of terminal commands to be executed"
                      }
                    },
                    "required": ["commands"]
                  }
                }
                ],
                "function_call":{"name": "get_commands"}
            }))
            .send()
            .await?;
        println!("{:?}", response.text().await?);
        // let gpt_response = response.text().await?;
        // println!("{:?}", gpt_response);
        Ok(())
    }
    pub async fn read_file(&self, filepath: &str) -> String {
        //Maybe instead create a handler for agent / io relationships
        let file_contents = fs::read_to_string(filepath).expect("Couldn't read that boi");
        let prompt = format!("Summarize this code: {}", file_contents);

        let response = self.prompt(&prompt).await.unwrap();
        response.choices[0].message.content.clone()
    }
}


#[allow(unused)]
pub fn explain_execute_or_generate(prompt: String) {
    todo!("Find the 'sentiment' of the prompt")
}
