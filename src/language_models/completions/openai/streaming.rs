use super::super::streaming::{CompletionStreamStatus, StreamResponse};
use serde::Deserialize;

impl StreamResponse for OpenAiStreamResponse {}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiStreamResponse {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize, Clone)]
struct StreamChoice {
    pub delta: StreamDelta,
}

#[derive(Debug, Deserialize, Clone)]
struct StreamDelta {
    pub role: Option<String>,
    pub content: Option<String>,
}

impl Into<CompletionStreamStatus> for OpenAiStreamResponse {
    fn into(self) -> CompletionStreamStatus {
        match self.choices[0].delta.content.to_owned() {
            Some(response) => CompletionStreamStatus::Working(
                response
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .to_string(),
            ),
            None => CompletionStreamStatus::Finished,
        }
    }
}
