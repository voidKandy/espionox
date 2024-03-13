use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::language_models::ModelEndpointError;

use super::OpenAiUsage;

#[derive(Debug, Deserialize, Clone)]
pub enum OpenAiEmbeddingModel {
    Small,
    Large,
    Ada,
}

impl ToString for OpenAiEmbeddingModel {
    fn to_string(&self) -> String {
        String::from(match self {
            Self::Small => "text-embedding-3-small",
            Self::Large => "text-embedding-3-large",
            Self::Ada => "text-embedding-ada-002",
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
    object: String,
    index: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
    usage: OpenAiUsage,
}

#[tracing::instrument(name = "Get embedding from openai's endpoint")]
pub async fn get_embedding(
    client: &Client,
    api_key: &str,
    text: &str,
    model: OpenAiEmbeddingModel,
) -> Result<OpenAiEmbeddingResponse, ModelEndpointError> {
    let payload = json!({ "input": text, "model": &model.to_string()});
    tracing::info!("Payload: {:?}", payload);
    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .unwrap();
    tracing::info!("Response: {:?}", response);
    let response = response.json().await?;
    Ok(response)
    // Err(ModelEndpointError::Recoverable)
}
