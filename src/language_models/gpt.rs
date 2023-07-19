use crate::agent::functions::config::Function;
use reqwest::Client;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct GptResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    pub content: Option<String>,
    pub function_call: Option<Value>,
}

#[derive(Clone)]
pub struct Gpt {
    pub config: GptConfig,
}

#[derive(Clone)]
pub struct GptConfig {
    api_key: String,
    client: Client,
    url: String,
    pub system_message: String,
}

impl GptResponse {
    pub fn parse_response(&self) -> Result<String, Box<dyn Error>> {
        println!("{:?}", &self);
        match self.choices[0].message.content.to_owned() {
            Some(response) => Ok(response),
            None => Err("Unable to parse completion response".into()),
        }
    }
    pub fn parse_fn_response(&self, fn_name: &str) -> Result<Vec<String>, Box<dyn Error>> {
        match self
            .choices
            .to_owned()
            .into_iter()
            .next()
            .unwrap()
            .message
            .function_call
        {
            Some(response) => {
                // println!("{:?}", response);
                let args_json = response
                    .get("arguments")
                    .expect("Couldn't parse arguments")
                    .as_str()
                    .unwrap();

                let args_value = serde_json::from_str::<Value>(args_json)?;
                let commands = args_value[fn_name].as_array().unwrap();

                let command_strings: Vec<String> = commands
                    .iter()
                    .filter_map(|command| command.as_str().map(String::from))
                    .collect();

                Ok(command_strings)
            }
            None => Err("Unable to parse completion response".into()),
        }
    }
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
    pub fn init(sys_message: &str) -> Gpt {
        let config = GptConfig::init(sys_message.to_string());
        Gpt { config }
    }

    pub async fn completion(&self, context: &Vec<Value>) -> Result<GptResponse, Box<dyn Error>> {
        let model = env::var("GPT_MODEL").unwrap();
        let payload =
            json!({"model": model, "messages": context, "max_tokens": 1000, "n": 1, "stop": null});
        println!("   PAYLOAD\n_____________________\n\n{:?}\n", &payload);
        match self
            .config
            .client
            .post(&self.config.url.clone())
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                let gpt_response: GptResponse = response.json().await?;
                println!("{:?}", gpt_response);
                Ok(gpt_response)
            }
            Err(err) => {
                println!("Completion Error: {err:?}");
                Err(err.into())
            }
        }
    }

    pub async fn function_completion(
        &self,
        context: &Vec<Value>,
        function: &Function,
    ) -> Result<GptResponse, Box<dyn Error>> {
        let functions_json: Value = serde_json::from_str(&function.render()).unwrap();
        let model = env::var("GPT_MODEL").unwrap();
        let payload = json!({
            "model": model,
            "messages": context,
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
        // println!("{:?}", &response.text().await);
        let gpt_response = response.json().await?;
        Ok(gpt_response)
        // Err("tst".into())
    }
}
