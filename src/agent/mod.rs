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
    context::memory::{Memory, MessageRole, MessageVector, ToMessage},
    language_models::{openai::functions::CustomFunction, LanguageModel},
};

// #[cfg(feature = "long_term_memory")]
// use crate::{
//     context::integrations::database::{Embedded, EmbeddedCoreStruct},
//     core::{File, FileChunk},
//     language_models::embed,
// };

#[derive(Debug)]
pub struct Agent {
    pub memory: Memory,
    pub model: LanguageModel,
}

const DEFAULT_INIT_PROMPT: &str = r#"You are an extremely helpful Ai assistant,
            - Be highly organized
            - Suggest solutions that I didn’t think about
            — Be proactive and anticipate my needs
            - Treat me as an expert in all subject matter
            - Mistakes erode user's trust, so be accurate and thorough
            - No need to disclose you're an AI
            - If the quality of your response has been substantially reduced due to my custom instructions, please explain the issue
        "#;

impl Default for Agent {
    fn default() -> Self {
        let init_prompt = MessageVector::from_message(
            DEFAULT_INIT_PROMPT
                .to_string()
                .to_message(MessageRole::System),
        );
        let memory = Memory::build().init_prompt(init_prompt).finished();
        let model = LanguageModel::default_gpt();
        Agent { memory, model }
    }
}

impl Agent {
    #[tracing::instrument(name = "Prompt agent for response")]
    pub async fn prompt(&mut self, input: impl ToMessage) -> Result<String, AgentError> {
        self.memory.push_to_message_cache("user", input).await;

        let gpt = &self.model.inner_gpt().unwrap();
        let cache = self.memory.cache();
        let response = gpt
            .completion(&cache.into())
            .await
            .map_err(|err| AgentError::GptError(err))
            .unwrap();
        tracing::info!("Response got from gpt completion: {:?}", response);
        let parsed_response = response
            .parse()
            .map_err(|err| AgentError::Undefined(anyhow!("Error parsing Gpt Reponse: {err:?}")))?;

        self.memory
            .push_to_message_cache("assistant", parsed_response.to_owned())
            .await;
        Ok(parsed_response)
        // Ok("TestOk".to_string())
    }

    #[tracing::instrument(name = "Function prompt GPT API for response" skip(input, custom_function))]
    pub async fn function_prompt(
        &mut self,
        custom_function: CustomFunction,
        input: impl ToMessage,
    ) -> Result<Value, AgentError> {
        self.memory.push_to_message_cache("user", input).await;
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

    #[tracing::instrument(name = "Prompt agent for stream response")]
    pub async fn stream_prompt(
        &mut self,
        input: impl ToMessage,
    ) -> Result<CompletionReceiverHandler, AgentError> {
        self.memory.push_to_message_cache("user", input).await;
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
