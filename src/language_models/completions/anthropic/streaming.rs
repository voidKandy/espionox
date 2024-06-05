use crate::language_models::completions::streaming::{CompletionStreamStatus, StreamResponse};
use serde::Deserialize;

impl StreamResponse for AnthropicStreamResponse {}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum AnthropicStreamResponse {
    #[serde(rename = "message_start")]
    MessageStart { message: Message },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: usize,
        content_block: ContentBlock,
    },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: Delta },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: MessageDelta, usage: Usage },
    #[serde(rename = "message_stop")]
    MessageStop,
}

#[derive(Debug, Deserialize, Clone)]
struct Message {
    id: String,
    #[serde(rename = "type")]
    msg_type: String,
    role: String,
    content: Vec<String>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: Usage,
}

#[derive(Debug, Deserialize, Clone)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
enum Delta {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
}

impl Delta {
    fn inner_text(self) -> String {
        match self {
            Self::TextDelta { text } => text,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct MessageDelta {
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl Into<CompletionStreamStatus> for AnthropicStreamResponse {
    fn into(self) -> CompletionStreamStatus {
        match self {
            Self::MessageStop | Self::ContentBlockStop { .. } => CompletionStreamStatus::Finished,
            Self::ContentBlockDelta { delta, .. } => {
                return CompletionStreamStatus::Working(delta.inner_text());
            }
            _ => CompletionStreamStatus::Working("".to_string()),
        }
    }
}
