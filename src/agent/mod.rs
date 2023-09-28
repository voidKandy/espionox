pub mod errors;
pub mod settings;
pub mod spo_agents;
pub mod streaming_utils;

use anyhow::anyhow;
pub use errors::AgentError;
pub use settings::AgentSettings;
pub use streaming_utils::*;

#[cfg(feature = "long_term_memory")]
use crate::{
    context::integrations::database::{Embedded, EmbeddedCoreStruct},
    core::{File, FileChunk},
    language_models::embed,
};

use crate::{
    context::{integrations::core::BufferDisplay, Context},
    language_models::openai::{functions::CustomFunction, gpt::Gpt},
};
use serde_json::Value;

#[derive(Debug)]
pub struct Agent {
    pub context: Context,
    gpt: Gpt,
}

impl Default for Agent {
    fn default() -> Self {
        Agent::build(AgentSettings::default()).expect("Failed to build default agent")
    }
}

impl Agent {
    pub fn build(settings: AgentSettings) -> anyhow::Result<Agent> {
        let gpt = Gpt::default();
        let context = Context::from_settings(settings);
        Ok(Agent { gpt, context })
    }

    #[tracing::instrument(name = "Prompt GPT API for response")]
    pub async fn prompt(&mut self, input: &impl BufferDisplay) -> Result<String, AgentError> {
        self.context.push_to_buffer("user", input);

        let gpt = self.gpt.clone();
        let buffer = self.context.buffer().clone();
        let response = gpt
            .completion(&buffer.into())
            .await
            .map_err(|err| AgentError::GptError(err))?
            .parse()
            .map_err(|err| AgentError::Undefined(anyhow!("Error parsing Gpt Reponse: {err:?}")))?;

        self.context.push_to_buffer("assistant", &response);
        Ok(response)
        // match rx.try_recv().unwrap() {
        //     Ok(response) => {
        //         let result = response
        //             .parse()
        //             .expect("Failed to parse completion response");
        //         self.context.push_to_buffer("assistant", &result);
        //
        //         Ok(result)
        //     }
        //     Err(err) => Err(err),
        // }
    }

    #[tracing::instrument(name = "Function prompt GPT API for response" skip(input, custom_function))]
    pub async fn function_prompt(
        &mut self,
        custom_function: CustomFunction,
        input: &impl BufferDisplay,
    ) -> Result<Value, AgentError> {
        self.context.push_to_buffer("user", input);
        let func = custom_function.function();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer().clone();
        tracing::info!("Buffer payload: {:?}", buffer);
        let function_response = gpt
            .function_completion(&buffer.into(), &func)
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
        input: &impl BufferDisplay,
    ) -> Result<CompletionReceiverHandler, AgentError> {
        self.context.push_to_buffer("user", input);
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer().clone();
        let response_stream = gpt.stream_completion(&buffer.into()).await?;

        let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
            tokio::sync::mpsc::channel(50);
        CompletionStreamingThread::spawn_poll_stream_for_tokens(response_stream, tx);
        Ok(CompletionReceiverHandler::from(rx))
    }

    #[cfg(feature = "long_term_memory")]
    pub fn vector_query_files(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
        use crate::context::long_term::LongTermMemory;

        match &self.context.long_term {
            LongTermMemory::Some(mem) => {
                let query_vector = embed(query).expect("Failed to embed query");
                Some(File::get_from_embedding(query_vector.into(), &mem.pool()))
            }
            _ => None,
        }
    }

    #[cfg(feature = "long_term_memory")]
    pub fn vector_query_chunks(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
        use crate::context::long_term::LongTermMemory;

        match &self.context.long_term {
            LongTermMemory::Some(mem) => {
                let query_vector = embed(query).expect("Failed to embed query");
                Some(FileChunk::get_from_embedding(
                    query_vector.into(),
                    &mem.pool(),
                ))
            }
            _ => None,
        }
    }
}
