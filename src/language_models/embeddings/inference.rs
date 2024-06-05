use super::error::EmbeddingResult;
use futures::Future;
use reqwest::{header::HeaderMap, Response};
use serde_json::Value;
use std::{fmt::Debug, pin::Pin};

pub type ProcessEmbeddingResponseReturn<'r> =
    Pin<Box<dyn Future<Output = EmbeddingResult<Vec<f32>>> + Send + Sync + 'r>>;
pub trait EmbeddingRequest: Debug + Sync + Send + 'static {
    fn headers(&self, api_key: &str) -> HeaderMap;
    fn model_str(&self) -> &str;
    fn url_str(&self) -> &str;
    fn as_json(&self, text: &str) -> EmbeddingResult<Value>;
    fn process_response<'r>(&'r self, response: Response) -> ProcessEmbeddingResponseReturn;
}
