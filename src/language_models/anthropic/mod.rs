use reqwest::Client;
use serde_json::{json, Value};

use super::error::ModelEndpointError;

pub static ANTHROPIC_COMPLETION_URL: &str = "https://api.anthropic.com/v1/messages";

pub enum AnthropicCompletionModel {
    Opus,
    Sonnet,
    Haiku,
}

impl ToString for AnthropicCompletionModel {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Opus => "claude-3-opus-20240229",
            Self::Sonnet => "claude-3-sonnet-20240229",
            Self::Haiku => "claude-3-haiku-20240307",
        })
    }
}

#[tracing::instrument(name = "Get completion", skip(client, api_key, model))]
pub(crate) async fn io_completion(
    client: &Client,
    api_key: &str,
    context: &Vec<Value>,
    model: &AnthropicCompletionModel,
) -> Result<(), ModelEndpointError> {
    // let gpt = model.inner_gpt().unwrap();
    // let temperature = (gpt.temperature * 10.0).round() / 10.0;
    // let payload = json!({"model": gpt.model.to_string(), "messages": context, "temperature": temperature, "max_tokens": 1000, "n": 1, "stop": null});
    // let response = client
    //     .post(ANTHROPIC_COMPLETION_URL)
    //     .header("Authorization", format!("Bearer {}", api_key))
    //     .header("Content-Type", "application/json")
    //     .json(&payload)
    //     .send()
    //     .await?;
    // let gpt_response = response.json().await?;
    // Ok()
    Ok(())
}
