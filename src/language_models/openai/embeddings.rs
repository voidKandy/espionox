use std::fmt::Display;

use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::language_models::{
    error::InferenceHandlerError,
    inference::{EmbeddingEndpointHandler, InferenceEndpointHandler},
};

use super::OpenAiUsage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenAiEmbeddingModel {
    Small,
    Large,
    Ada,
}

impl Display for OpenAiEmbeddingModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiEmbeddingData {
    pub embedding: Vec<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiEmbeddingResponse {
    pub data: Vec<OpenAiEmbeddingData>,
    pub usage: OpenAiUsage,
}

impl InferenceEndpointHandler for OpenAiEmbeddingModel {
    fn name(&self) -> &str {
        match self {
            Self::Small => "text-embedding-3-small",
            Self::Large => "text-embedding-3-large",
            Self::Ada => "text-embedding-ada-002",
        }
    }
    fn completion_url(&self) -> &str {
        "https://api.openai.com/v1/embeddings"
    }
    fn request_headers(&self, api_key: &str) -> reqwest::header::HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        map.insert("Content-Type", "application/json".parse().unwrap());
        map
    }
}

impl EmbeddingEndpointHandler for OpenAiEmbeddingModel {
    fn request_body(&self, text: &str) -> serde_json::Value {
        json!({ "input": text, "model": self.name()})
    }
    fn handle_response(
        &self,
        response: serde_json::Value,
    ) -> Result<Vec<f32>, InferenceHandlerError> {
        let response: OpenAiEmbeddingResponse = serde_json::from_value(response)?;
        Ok(response.data[0].embedding.to_owned())
    }
}
