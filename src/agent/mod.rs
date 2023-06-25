pub mod gpt_complete;
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
    pub async fn send_prompt(
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
    pub async fn read_file(&self, filepath: &str) -> String {
        let file_contents = fs::read_to_string(filepath).expect("Couldn't read that boi");
        let prompt = format!("Summarize this code: {}", file_contents);

        let response = self.send_prompt(&prompt).await.unwrap();
        response.choices[0].message.content.clone()
    }
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

#[allow(unused)]
pub fn explain_execute_or_generate(prompt: String) {
    todo!("Find the 'sentiment' of the prompt")
}
