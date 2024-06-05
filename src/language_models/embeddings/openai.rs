use super::inference::EmbeddingRequest;
use crate::language_models::completions::openai::requests::OpenAiUsage;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OpenAiEmbeddingModel {
    Small,
    Large,
    #[default]
    Ada,
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

impl EmbeddingRequest for OpenAiEmbeddingModel {
    fn model_str(&self) -> &str {
        match self {
            Self::Small => "text-embedding-3-small",
            Self::Large => "text-embedding-3-large",
            Self::Ada => "text-embedding-ada-002",
        }
    }
    fn url_str(&self) -> &str {
        "https://api.openai.com/v1/embeddings"
    }
    fn headers(&self, api_key: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        map.insert(
            "Authorization",
            format!("Bearer {}", api_key).parse().unwrap(),
        );
        map.insert("Content-Type", "application/json".parse().unwrap());
        map
    }
    fn as_json(&self, text: &str) -> super::error::EmbeddingResult<serde_json::Value> {
        Ok(json!({ "input": text, "model": self.model_str()}))
    }
    fn process_response<'r>(
        &'r self,
        response: reqwest::Response,
    ) -> super::inference::ProcessEmbeddingResponseReturn {
        Box::pin(async {
            let json = response.json().await?;
            let response: OpenAiEmbeddingResponse = serde_json::from_value(json)?;
            Ok(response.data[0].embedding.to_owned())
        })
    }
}
