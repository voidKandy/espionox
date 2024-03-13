pub mod models;
pub mod streaming;
use super::super::{super::LanguageModel, functions::Function};
use crate::language_models::error::ModelEndpointError;
use anyhow::anyhow;
pub use models::*;
use reqwest::Client;
use reqwest_streams::JsonStreamResponse;
use serde_json::{json, Value};
use std::pin::Pin;
use std::{future::Future, time::Duration};
use streaming::*;

pub fn io_completion_fn_wrapper<'c>(
    client: &'c Client,
    api_key: &'c str,
    context: &'c Vec<Value>,
    model: &'c LanguageModel,
) -> Pin<Box<dyn Future<Output = Result<OpenAiResponse, ModelEndpointError>> + Send + Sync + 'c>> {
    Box::pin(io_completion(client, api_key, context, model))
}

pub fn stream_completion_fn_wrapper<'c>(
    client: &'c Client,
    api_key: &'c str,
    context: &'c Vec<Value>,
    model: &'c LanguageModel,
) -> Pin<Box<dyn Future<Output = Result<CompletionStream, ModelEndpointError>> + Send + Sync + 'c>>
{
    Box::pin(stream_completion(client, api_key, context, model))
}

pub fn function_completion_fn_wrapper<'c>(
    client: &'c Client,
    api_key: &'c str,
    context: &'c Vec<Value>,
    model: &'c LanguageModel,
    function: &'c Function,
) -> Pin<Box<dyn Future<Output = Result<OpenAiResponse, ModelEndpointError>> + Send + Sync + 'c>> {
    Box::pin(function_completion(
        client, api_key, context, model, function,
    ))
}

#[tracing::instrument(name = "Get completion", skip(client, api_key, model))]
pub(crate) async fn io_completion(
    client: &Client,
    api_key: &str,
    context: &Vec<Value>,
    model: &LanguageModel,
) -> Result<OpenAiResponse, ModelEndpointError> {
    let gpt = model.inner_gpt().unwrap();
    let temperature = (gpt.temperature * 10.0).round() / 10.0;
    let payload = json!({"model": gpt.model.to_string(), "messages": context, "temperature": temperature, "max_tokens": 1000, "n": 1, "stop": null});
    let response = client
        .post(model.completion_url())
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let gpt_response = response.json().await?;
    Ok(gpt_response)
}

#[tracing::instrument(name = "Get streamed completion", skip(client, api_key, model))]
pub async fn stream_completion(
    client: &Client,
    api_key: &str,
    context: &Vec<Value>,
    model: &LanguageModel,
) -> Result<CompletionStream, ModelEndpointError> {
    let gpt = model.inner_gpt().unwrap();
    let temperature = (gpt.temperature * 10.0).round() / 10.0;
    let payload = json!({
        "model": gpt.model.to_string(),
        "messages": context,
        "temperature": temperature,
        "stream": true,
        "max_tokens": 1000,
        "n": 1,
        "stop": null,
    });
    tracing::info!("Json payload: {:?}", &payload);

    let request = client
        .post(model.completion_url())
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload);
    tracing::info!("Request: {:?}", &request);
    let response_stream = tokio::time::timeout(Duration::from_secs(10), async {
        request
            .send()
            .await
            .map_err(|err| ModelEndpointError::NetRequest(err))
            .unwrap()
            .json_array_stream::<StreamResponse>(1024)
    })
    .await
    .map_err(|_| ModelEndpointError::Undefined(anyhow!("Response stream request timed out")))?;

    tracing::info!("Got response stream, returning");
    Ok(Box::new(response_stream))
}

#[tracing::instrument(name = "Get function completion" skip(context, function))]
pub async fn function_completion(
    client: &Client,
    api_key: &str,
    context: &Vec<Value>,
    model: &LanguageModel,
    function: &Function,
) -> Result<OpenAiResponse, ModelEndpointError> {
    let gpt = model.inner_gpt().unwrap();
    let payload = json!({
        "model": gpt.model.to_string(),
        "messages": context,
        "functions": [function.json],
        "function_call": {"name": function.name}
    });
    tracing::info!("Full completion payload: {:?}", payload);
    let response = client
        .post(model.completion_url())
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let gpt_response = response.json().await?;
    Ok(gpt_response)
}

impl OpenAiResponse {
    #[tracing::instrument(name = "Parse gpt response into string")]
    pub fn parse(&self) -> Result<String, anyhow::Error> {
        match self.choices[0].message.content.to_owned() {
            Some(response) => Ok(response),
            None => Err(anyhow!("Unable to parse completion response")),
        }
    }

    #[tracing::instrument]
    pub fn parse_fn(&self) -> Result<Value, anyhow::Error> {
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
                tracing::info!("Function response: {:?}", response);
                let args_json = serde_json::from_str::<Value>(
                    response
                        .get("arguments")
                        .expect("Couldn't parse arguments")
                        .as_str()
                        .unwrap(),
                )?;

                tracing::info!("Args json: {:?}", args_json);
                let mut args_output: Value = json!({});
                if let Some(arguments) = args_json.as_object() {
                    for (key, value) in arguments.iter() {
                        args_output
                            .as_object_mut()
                            .expect("Failed to get array mut")
                            .insert(key.to_string(), value.clone());
                    }
                }
                tracing::info!("Args output: {:?}", args_output);
                Ok(args_output)
            }
            None => Err(anyhow!("Unable to parse completion response")),
        }
    }
}
