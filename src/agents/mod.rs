pub mod error;
pub mod independent;
pub mod memory;
use dotenv::dotenv;
use memory::MessageStack;

use crate::language_models::LanguageModel;
use anyhow::anyhow;
pub use error::AgentError;

use crate::language_models::openai::endpoints::completions::OpenAiResponse;

/// Agent struct for interracting with LLM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    /// Unique Identifier
    /// Memory contains cache and ltm
    pub cache: MessageStack,
    /// Language model defines which model to use for the given agent
    pub model: LanguageModel,
}

impl Default for Agent {
    fn default() -> Self {
        dotenv().ok();

        let prompt = std::env::var("DEFAULT_INIT_PROMPT");
        let cache = match prompt.ok() {
            Some(p) => MessageStack::new(&p),
            None => MessageStack::init(),
        };

        tracing::info!("Default Agent initialized with cache: {:?}", cache);
        let model = LanguageModel::default_openai();
        Agent { cache, model }
    }
}

impl Agent {
    /// Helper function for creating an Agent given system prompt content and model
    pub fn new(init_prompt: &str, model: LanguageModel) -> Self {
        let cache = MessageStack::new(init_prompt);
        Agent {
            cache,
            model,
            ..Default::default()
        }
    }

    #[tracing::instrument(name = "Parse OpenAiResponse and add token count")]
    pub fn handle_completion_response(
        &mut self,
        response: OpenAiResponse,
    ) -> Result<String, AgentError> {
        let gpt = self.model.inner_mut_gpt().unwrap();
        gpt.token_count += response.usage.total_tokens;

        tracing::info!(
            "{} tokens added to model token count. Total count: {}",
            response.usage.total_tokens,
            gpt.token_count
        );

        let parsed_response = response
            .parse()
            .map_err(|err| AgentError::Undefined(anyhow!("Error parsing Gpt Reponse: {err:?}")))?;

        Ok(parsed_response)
    }
}
