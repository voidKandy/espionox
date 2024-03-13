pub mod error;
pub mod independent;
pub mod language_models;
pub mod memory;
use dotenv::dotenv;
use memory::MessageStack;

use anyhow::anyhow;
pub use error::AgentError;
use language_models::LanguageModel;

use self::language_models::openai::gpt::GptResponse;

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
        let model = LanguageModel::default_gpt();
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

    #[tracing::instrument(name = "Parse GptResponse and add token count")]
    pub fn handle_completion_response(
        &mut self,
        response: GptResponse,
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
