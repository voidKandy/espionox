use super::fn_render::Function;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::error::Error;

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
    pub content: Option<String>,
    pub function_call: Option<Value>,
}

#[derive(Clone)]
pub struct Gpt {
    pub config: GptConfig,
    pub permissions: GptPermissions,
}

#[derive(Clone)]
pub struct GptPermissions {
    pub write: bool,
    pub read: bool,
    pub execute: bool,
}

#[derive(Clone)]
pub struct GptConfig {
    api_key: String,
    client: Client,
    url: String,
    pub system_message: String,
}

impl GptConfig {
    pub fn init(system_message: String) -> GptConfig {
        dotenv::dotenv().ok();
        let api_key = env::var("OPEN_AI_API_KEY").unwrap();
        let client = Client::new();
        let url = "https://api.openai.com/v1/chat/completions".to_string();
        GptConfig {
            api_key,
            client,
            url,
            system_message,
        }
    }
}

impl Gpt {
    pub fn init(sys_message: String) -> Gpt {
        let config = GptConfig::init(sys_message);
        let permissions = GptPermissions {
            write: true,
            read: true,
            execute: true,
        };
        Gpt {
            config,
            permissions,
        }
    }

    // Create something to handle 'Context'
    pub async fn completion(&self, prompt: &str) -> Result<GptResponse, Box<dyn Error>> {
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

    pub async fn function_completion(
        &self,
        prompt: &str,
        function: &Function,
    ) -> Result<GptResponse, Box<dyn Error>> {
        let functions_json: Value = serde_json::from_str(&function.render()).unwrap();
        let payload = json!({
            "model": "gpt-3.5-turbo-0613",
            "messages": [
                {"role": "system", "content": self.config.system_message},
                {"role": "user", "content": prompt.to_string()}
            ],
            "functions": [functions_json],
            "function_call": {"name": function.name}
        });
        let response = self
            .config
            .client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;
        let gpt_response = response.json().await?;
        Ok(gpt_response)
    }
}
