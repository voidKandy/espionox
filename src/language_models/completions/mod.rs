pub mod anthropic;
pub mod error;
pub mod functions;
#[cfg(feature = "bert")]
pub mod huggingface;
mod inference;
pub mod openai;
pub mod streaming;
use self::{
    anthropic::builder::AnthropicCompletionModel, error::CompletionResult, functions::Function,
    inference::CompletionRequestBuilder, openai::builder::OpenAiCompletionModel,
    streaming::ProviderStreamHandler,
};

use crate::agents::memory::MessageStack;
use anyhow::anyhow;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use tracing::{info, warn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionProvider {
    OpenAi(OpenAiCompletionModel),
    Anthropic(AnthropicCompletionModel),
}

impl From<OpenAiCompletionModel> for CompletionProvider {
    fn from(value: OpenAiCompletionModel) -> Self {
        Self::OpenAi(value)
    }
}

impl From<AnthropicCompletionModel> for CompletionProvider {
    fn from(value: AnthropicCompletionModel) -> Self {
        Self::Anthropic(value)
    }
}

impl CompletionProvider {
    fn inner_builder(&self) -> Box<&dyn CompletionRequestBuilder> {
        match &self {
            Self::OpenAi(b) => return Box::new(b),
            Self::Anthropic(b) => return Box::new(b),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionModel {
    provider: CompletionProvider,
    params: ModelParameters,
    api_key: String,
    #[serde(skip)]
    client: Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParameters {
    /// Total token usage count of the model
    pub total_token_count: u32,
    /// What sampling temperature to use, between 0 and 2.
    /// Higher values like 0.8 will make the output more random,
    /// while lower values like 0.2 will make it more focused and deterministic.
    /// Input as a value between 0 and 200. Will be turned into float.
    pub temperature: Option<u8>,
    /// Number between -2.0 and 2.0.
    /// Positive values penalize new tokens based on their existing frequency in the text so far,
    /// decreasing the model's likelihood to repeat the same line verbatim.
    pub frequency_penalty: Option<i8>,
    /// The maximum number of tokens that can be generated in the chat completion.
    /// The total length of input tokens and generated tokens is limited by the model's context length.
    pub max_tokens: Option<u32>,
    /// How many chat completion choices to generate for each input message.
    /// Note that you will be charged based on the number of generated tokens across all of the choices.
    /// Keep n as 1 to minimize costs.
    pub n: Option<u32>,
    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far,
    /// increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<i8>,
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            total_token_count: 0,
            temperature: Some(70),
            frequency_penalty: None,
            max_tokens: None,
            n: Some(1),
            presence_penalty: None,
        }
    }
}

impl ModelParameters {
    /// Temperature is computed to a number between 0 and 1 by dividing this value by 100
    fn temperature(&self) -> Result<f32, anyhow::Error> {
        Ok((self.temperature.ok_or(anyhow!("No temperature"))? / 100) as f32)
    }
}

impl CompletionModel {
    pub fn new(
        m: impl Into<CompletionProvider>,
        params: ModelParameters,
        api_key: &str,
    ) -> CompletionModel {
        let client = Client::new();
        Self {
            provider: m.into(),
            params,
            client,
            api_key: api_key.to_owned(),
        }
    }

    ///  openai gpt3 handler with 0.7 temp
    pub fn default_openai(api_key: &str) -> CompletionModel {
        let provider = CompletionProvider::OpenAi(OpenAiCompletionModel::default());
        let client = reqwest::Client::new();
        CompletionModel {
            provider,
            params: ModelParameters::default(),
            api_key: api_key.to_owned(),
            client,
        }
    }

    ///  anthropic Haiku handler with 0.7 temp
    pub fn default_anthropic(api_key: &str) -> CompletionModel {
        let provider = CompletionProvider::Anthropic(AnthropicCompletionModel::default());
        let client = reqwest::Client::new();
        CompletionModel {
            provider,
            params: ModelParameters::default(),
            api_key: api_key.to_owned(),
            client,
        }
    }

    #[tracing::instrument(name = "io completion", skip_all)]
    pub(crate) async fn get_io_completion(
        &self,
        messages: &MessageStack,
    ) -> CompletionResult<String> {
        let builder = self.provider.inner_builder();
        let headers = builder.headers(&self.api_key);
        let url = builder.url_str();
        let req = builder.into_io_req(messages, &self.params)?;
        let json_req = req.as_json()?;
        info!(
            "\nSending request:\n{:?}\nto: {}\nwith headers: {:?}\n",
            json_req, url, headers
        );

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&json_req)
            .send()
            .await?;
        match req.process_response(response).await {
            Ok(r) => return Ok(TryInto::<String>::try_into(r)?),
            Err(err) => {
                warn!("Error getting Io completion: {:?}", err);
                Err(err)
            }
        }
    }

    #[tracing::instrument(name = "streamed completion", skip_all)]
    pub(crate) async fn get_stream_completion(
        &self,
        messages: &MessageStack,
    ) -> CompletionResult<ProviderStreamHandler> {
        let builder = self.provider.inner_builder();
        let headers = builder.headers(&self.api_key);
        let url = builder.url_str();
        let req = builder.into_stream_req(messages, &self.params)?;
        let json_req = req.as_json()?;
        info!(
            "\nSending request:\n{:?}\nto: {}\nwith headers: {:?}\n",
            json_req, url, headers
        );

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&json_req)
            .send()
            .await?;

        match req.process_response(response).await {
            Ok(r) => return Ok(TryInto::<ProviderStreamHandler>::try_into(r)?),
            Err(err) => {
                warn!("Error getting streamed Io completion: {:?}", err);
                Err(err.into())
            }
        }
    }

    #[tracing::instrument(name = "function completion", skip_all)]
    pub(crate) async fn get_fn_completion(
        &self,
        messages: &MessageStack,
        function: Function,
    ) -> CompletionResult<Value> {
        let builder = self.provider.inner_builder();
        let headers = builder.headers(&self.api_key);
        let url = builder.url_str();
        let req = builder.serialize_function(messages, function)?;
        info!(
            "\nSending request:\n{:?}\nto: {}\nwith headers: {:?}\n",
            req, url, headers
        );

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&req)
            .send()
            .await?;

        info!("Got response: {:?}", response);
        match builder.process_function_response(response.json().await?) {
            Ok(r) => return Ok(r),
            Err(err) => {
                warn!("Error getting function completion: {:?}", err);
                Err(err.into())
            }
        }
    }
}
