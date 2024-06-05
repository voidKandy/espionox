use self::{error::EmbeddingResult, inference::EmbeddingRequest, openai::OpenAiEmbeddingModel};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod inference;
pub mod openai;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingProvider {
    OpenAi(OpenAiEmbeddingModel),
}

impl EmbeddingProvider {
    fn inner_request(&self) -> Box<&dyn EmbeddingRequest> {
        match &self {
            Self::OpenAi(b) => return Box::new(b),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModel {
    provider: EmbeddingProvider,
    api_key: String,
    #[serde(skip)]
    client: Client,
}

impl EmbeddingModel {
    pub fn default_openai(api_key: &str) -> Self {
        let client = Client::new();
        Self {
            provider: EmbeddingProvider::OpenAi(OpenAiEmbeddingModel::default()),
            api_key: api_key.to_owned(),
            client,
        }
    }

    pub async fn get_embedding(&self, text: &str) -> EmbeddingResult<Vec<f32>> {
        let request = self.provider.inner_request();
        let headers = request.headers(&self.api_key);
        let url = request.url_str();
        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request.as_json(text)?)
            .send()
            .await?;
        Ok(request.process_response(response).await?)
    }
}
