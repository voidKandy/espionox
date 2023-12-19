pub mod completions;
pub mod config;
pub mod error;
pub mod streaming_utils;

use crate::configuration::ConfigEnv;
pub use config::*;
pub use error::GptError;
pub use streaming_utils::*;

use super::functions::config::Function;
use anyhow::anyhow;
use bytes::Bytes;
#[allow(unused)]
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;

impl Default for Gpt {
    fn default() -> Self {
        let config = GptConfig::init(ConfigEnv::default());
        let model_override = None;
        let temperature = 0.7;
        let token_count = 0;
        Gpt {
            config,
            model_override,
            temperature,
            token_count,
        }
    }
}

impl Gpt {
    pub fn new(model: GptModel, temperature: f32) -> Self {
        let config = GptConfig::init(ConfigEnv::default());
        let model_override = Some(model);
        let token_count = 0;
        Self {
            config,
            model_override,
            temperature,
            token_count,
        }
    }
    fn model_string(&self) -> String {
        match &self.model_override {
            Some(model) => model.to_string(),
            None => self.config.model.to_string(),
        }
    }
}
impl GptConfig {
    pub fn init(env: ConfigEnv) -> GptConfig {
        let settings = env
            .global_settings()
            .expect("Failed to get model settings")
            .language_model;
        let api_key = settings.api_key;
        let model = GptModel::default();
        let client = Client::new();
        let url = "https://api.openai.com/v1/chat/completions".to_string();
        GptConfig {
            api_key,
            client,
            url,
            model,
        }
    }
}
