pub mod errors;
// pub mod settings;
pub mod spo_agents;
pub mod streaming_utils;
//
use anyhow::anyhow;
pub use errors::AgentError;
use serde_json::Value;
pub use streaming_utils::*;

use crate::{
    language_models::{openai::functions::CustomFunction, LanguageModel},
    memory::{Memory, ToMessage},
};

/// Agent struct for interracting with LLM
#[derive(Debug, Clone)]
pub struct Agent {
    /// Memory handles how agent recalls and caches memory
    pub memory: Memory,
    /// Language model defines which model to use for the given agent
    pub model: LanguageModel,
}

impl Default for Agent {
    fn default() -> Self {
        let init_prompt = crate::persistance::prompts::get_prompt_by_name("DEFAULT_INIT_PROMPT")
            .expect("Failed to get default init prompt");
        let memory = Memory::build().init_prompt(init_prompt).finished();
        let model = LanguageModel::default_gpt();
        Agent { memory, model }
    }
}

impl Agent {
    /// This method does 3 things:
    /// * Uses LanguageModel enum to get a completion response
    /// * Updates gpt token_count
    /// * returns response as a string
    #[tracing::instrument(name = "Prompt agent for response")]
    pub async fn prompt(&mut self, input: impl ToMessage) -> Result<String, AgentError> {
        self.memory.push_to_message_cache(Some("user"), input).await;

        let gpt = self.model.inner_gpt().unwrap();
        let cache = self.memory.cache();
        let response = gpt
            .completion(&cache.into())
            .await
            .map_err(|err| AgentError::GptError(err))
            .unwrap();
        tracing::info!("Response got from gpt completion: {:?}", response);

        gpt.token_count += response.usage.total_tokens;

        tracing::info!(
            "{} tokens added to model token count. Total count: {}",
            response.usage.total_tokens,
            gpt.token_count
        );

        let parsed_response = response
            .parse()
            .map_err(|err| AgentError::Undefined(anyhow!("Error parsing Gpt Reponse: {err:?}")))?;

        self.memory
            .push_to_message_cache(Some("assistant"), parsed_response.to_owned())
            .await;
        Ok(parsed_response)
    }

    /// Openai function calling completion
    #[tracing::instrument(name = "Function prompt GPT API for response" skip(input, custom_function))]
    pub async fn function_prompt(
        &mut self,
        custom_function: CustomFunction,
        input: impl ToMessage,
    ) -> Result<Value, AgentError> {
        self.memory.push_to_message_cache(Some("user"), input).await;
        let func = custom_function.function();
        let gpt = &self.model.inner_gpt().unwrap();
        let cache = self.memory.cache();
        let function_response = gpt
            .function_completion(&cache.into(), &func)
            .await
            .map_err(|err| AgentError::GptError(err))?;
        tracing::info!("Function response: {:?}", function_response);
        Ok(function_response
            .parse_fn()
            .expect("failed to parse response"))
    }

    /// Get streamed completion, this function returns a reciever which must be tried to recieve
    /// tokens
    #[tracing::instrument(name = "Prompt agent for stream response")]
    pub async fn stream_prompt(
        &mut self,
        input: impl ToMessage,
    ) -> Result<CompletionReceiverHandler, AgentError> {
        self.memory.push_to_message_cache(Some("user"), input).await;
        let gpt = &self.model.inner_gpt().unwrap();
        let cache = self.memory.cache();
        let response_stream = gpt.stream_completion(&cache.into()).await?;

        let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
            tokio::sync::mpsc::channel(50);
        CompletionStreamingThread::spawn_poll_stream_for_tokens(response_stream, tx);
        Ok(CompletionReceiverHandler::from(rx))
    }
    //
    //     #[cfg(feature = "long_term_memory")]
    //     pub fn vector_query_files(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
    //         match &self.memory.long_term {
    //             LongTermMemory::Some(mem) => {
    //                 let query_vector = embed(query).expect("Failed to embed query");
    //                 Some(File::get_from_embedding(query_vector.into(), &mem.pool()))
    //             }
    //             _ => None,
    //         }
    //     }
    //
    //     #[cfg(feature = "long_term_memory")]
    //     pub fn vector_query_chunks(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
    //         match &self.context.long_term {
    //             LongTermMemory::Some(mem) => {
    //                 let query_vector = embed(query).expect("Failed to embed query");
    //                 Some(FileChunk::get_from_embedding(
    //                     query_vector.into(),
    //                     &mem.pool(),
    //                 ))
    //             }
    //             _ => None,
    //         }
    //     }
}
