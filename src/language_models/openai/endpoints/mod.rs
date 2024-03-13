pub mod completions;
pub mod embeddings;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct OpenAiUsage {
    prompt_tokens: i32,
    completion_tokens: Option<i32>,
    pub total_tokens: i32,
}
