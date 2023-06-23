use dotenv::dotenv;
use reqwest::Client;
use serde_derive::Deserialize;
#[allow(unused)]
use serde_json::json;
use std::env;
use std::fs;
use tokio;

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct GptResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct Message {
    content: String,
}

#[tokio::main]
#[allow(unused)]
pub async fn print_key() {
    dotenv().ok();
    let api_key = env::var("OPEN_AI_API_KEY").unwrap();
    let client = Client::new();
    let url = "https://api.openai.com/v1/chat/completions";
    let system_message = "You are an ai that summarizes code. Be as thorough as possible while also being as succinct as possible";

    let file_contents = fs::read_to_string("src/main.rs").expect("Couldn't read that boi");
    let prompt = format!("Summarize this code: {}", file_contents);

    let response = send_prompt(&client, url, &api_key, &prompt, system_message)
        .await
        .unwrap();
    println!("{}", response.choices[0].message.content)
}

async fn send_prompt(
    client: &Client,
    url: &str,
    api_key: &str,
    prompt: &str,
    system_message: &str,
) -> Result<GptResponse, Box<dyn std::error::Error>> {
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&json!({ "model": "gpt-3.5-turbo", "messages": [{"role": "system", "content": system_message}, {"role": "user", "content": prompt}], "max_tokens": 2000, "n": 1, "stop": null }))
        .send()
        .await?;

    let gpt_response: GptResponse = response.json().await?;
    Ok(gpt_response)
}
