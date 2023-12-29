use super::{super::functions::Function, models::*, streaming_utils::*, GptError};
use anyhow::anyhow;
use bytes::Bytes;
use reqwest_streams::JsonStreamResponse;
use serde_json::{json, Value};

const OPENAI_COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";

impl Gpt {
    #[tracing::instrument(name = "Get streamed completion")]
    pub async fn stream_completion(
        &self,
        context: &Vec<Value>,
    ) -> Result<CompletionStream, GptError> {
        let temperature = (self.temperature * 10.0).round() / 10.0;
        let payload = json!({
            "model": self.model.to_string(),
            "messages": context,
            "temperature": temperature,
            "stream": true,
            "max_tokens": 1000,
            "n": 1,
            "stop": null,
        });
        tracing::info!("PAYLOAD: {:?}", &payload);

        match &self.api_key {
            Some(key) => {
                let response_stream = self
                    .client
                    .post(OPENAI_COMPLETION_URL)
                    .header("Authorization", format!("Bearer {}", key))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|err| GptError::Completion(err))?
                    .json_array_stream::<StreamResponse>(1024);

                Ok(Box::new(response_stream))
            }
            None => Err(GptError::NoApiKey),
        }
    }

    #[tracing::instrument(name = "Get completion")]
    pub async fn completion(&self, context: &Vec<Value>) -> Result<GptResponse, GptError> {
        let temperature = (self.temperature * 10.0).round() / 10.0;
        let payload = json!({"model": self.model.to_string(), "messages": context, "temperature": temperature, "max_tokens": 1000, "n": 1, "stop": null});
        match &self.api_key {
            Some(key) => {
                let response = self
                    .client
                    .post(OPENAI_COMPLETION_URL)
                    .header("Authorization", format!("Bearer {}", key))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;
                let gpt_response = response.json().await?;
                Ok(gpt_response)
            }
            None => Err(GptError::NoApiKey),
        }
    }

    #[tracing::instrument(name = "Get function completion" skip(context, function))]
    pub async fn function_completion(
        &self,
        context: &Vec<Value>,
        function: &Function,
    ) -> Result<GptResponse, GptError> {
        let payload = json!({
            "model": self.model.to_string(),
            "messages": context,
            "functions": [function.json],
            "function_call": {"name": function.name}
        });
        tracing::info!("Full completion payload: {:?}", payload);
        match &self.api_key {
            Some(key) => {
                let response = self
                    .client
                    .post(OPENAI_COMPLETION_URL)
                    .header("Authorization", format!("Bearer {}", key))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await?;
                let gpt_response = response.json().await?;
                Ok(gpt_response)
            }
            None => Err(GptError::NoApiKey),
        }
    }
}

impl GptResponse {
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

impl StreamResponse {
    #[tracing::instrument(name = "Get token from byte chunk")]
    pub async fn from_byte_chunk(chunk: Bytes) -> Result<Option<Self>, GptError> {
        let chunk_string = String::from_utf8_lossy(&chunk).trim().to_string();

        let chunk_strings: Vec<&str> = chunk_string.split('\n').filter(|s| !s.is_empty()).collect();

        tracing::info!(
            "{} chunk data strings to process: {:?}",
            chunk_strings.len(),
            chunk_strings
        );
        for string in chunk_strings
            .iter()
            .map(|s| s.trim_start_matches("data:").trim())
        {
            tracing::info!("Processing string: {}", string);
            if string == "[DONE]" {
                return Ok(None);
            }

            match serde_json::from_str::<StreamResponse>(&string) {
                Ok(stream_response) => {
                    if let Some(choice) = &stream_response.choices.get(0) {
                        tracing::info!("Chunk as stream response: {:?}", stream_response);
                        if choice.delta.role.is_some() {
                            continue;
                        }
                        if choice.delta.content.is_none() && choice.delta.role.is_none() {
                            continue;
                        }
                        return Ok(Some(stream_response));
                    }
                }
                Err(err) => {
                    if err.to_string().contains("expected value") {
                        return Err(GptError::Recoverable(format!(
                            "Possibly recoverable error: {:?}",
                            err
                        )));
                    }
                }
            }
        }
        Ok(None)
    }

    #[tracing::instrument(name = "Parse stream response for string")]
    pub fn parse(&self) -> Result<String, anyhow::Error> {
        tracing::info!(
            "self.choices[0].delta.content: {}",
            self.choices[0]
                .delta
                .content
                .to_owned()
                .expect("Failed to get delta content")
        );
        match self.choices[0].delta.content.to_owned() {
            Some(response) => Ok(response
                .trim_start_matches('"')
                .trim_end_matches('"')
                .to_string()),
            None => Err(anyhow!("Unable to parse stream completion response")),
        }
    }
}
