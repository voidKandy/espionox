pub mod errors;
pub mod settings;
pub mod spo_agents;
pub mod streaming_utils;

pub use errors::AgentError;
pub use settings::AgentSettings;
pub use streaming_utils::*;

use crate::{
    configuration::ConfigEnv,
    context::{
        integrations::{
            core::BufferDisplay,
            database::{Embedded, EmbeddedCoreStruct},
        },
        Context, MemoryVariant,
    },
    core::{File, FileChunk},
    language_models::embed,
    language_models::openai::{functions::CustomFunction, gpt::Gpt},
};
use serde_json::Value;
use std::{sync::mpsc, thread};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Agent {
    pub context: Context,
    gpt: Gpt,
}

impl Default for Agent {
    fn default() -> Self {
        Agent::build(AgentSettings::default(), ConfigEnv::default())
            .expect("Failed to build default agent")
    }
}

impl Agent {
    pub fn build(settings: AgentSettings, env: ConfigEnv) -> anyhow::Result<Agent> {
        let gpt = Gpt::default();
        let mut context = match &settings.memory_override {
            Some(memory) => Context::build(memory.clone(), env),
            None => Context::build(MemoryVariant::default(), env),
        };

        match context.memory {
            MemoryVariant::Forget => context.buffer = settings.init_prompt,
            _ => {
                if context.buffer.len() == 0 {
                    context.buffer = settings.init_prompt;
                }
            }
        }

        Ok(Agent { gpt, context })
    }

    pub fn format_to_buffer(&mut self, o: impl BufferDisplay) {
        let mem = o.buffer_display();
        self.context.buffer.push_std("user", &mem);
    }

    pub fn switch_mem(&mut self, memory: MemoryVariant) {
        self.context.save_buffer();
        self.context = Context::build(memory, self.context.env.to_owned());
    }

    #[tracing::instrument(name = "Prompt GPT API for response")]
    pub fn prompt(&mut self, input: &str) -> String {
        self.context.buffer.push_std("user", &input);

        let (tx, rx) = mpsc::channel();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async move {
                gpt.completion(&buffer.into())
                    .await
                    .expect("Failed to get completion.")
            });
            tx.send(result).unwrap();
        })
        .join()
        .expect("Failed to join thread");
        let result = rx
            .recv()
            .unwrap()
            .parse()
            .expect("Failed to parse completion response");

        self.context.buffer.push_std("assistant", &result);
        result
    }

    #[tracing::instrument(name = "Function prompt GPT API for response" skip(input, custom_function))]
    pub fn function_prompt(&mut self, custom_function: CustomFunction, input: &str) -> Value {
        self.context.buffer.push_std("user", &input);
        let (tx, rx) = mpsc::channel();
        let func = custom_function.function();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        tracing::info!("Buffer payload: {:?}", buffer);

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async move {
                gpt.function_completion(&buffer.into(), &func)
                    .await
                    .expect("Failed to get completion.")
            });
            tx.send(result).unwrap();
        })
        .join()
        .expect("Failed to join thread");
        let function_response = rx.recv().unwrap();
        tracing::info!("Function response: {:?}", function_response);
        function_response
            .parse_fn()
            .expect("failed to parse response")
    }

    #[tracing::instrument(name = "Prompt agent for stream response")]
    pub async fn stream_prompt(&mut self, input: &str) -> CompletionReceiverHandler {
        self.context.buffer.push_std("user", &input);
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        let response_stream = gpt
            .stream_completion(&buffer.into())
            .await
            .expect("Failed to get completion.");

        let (tx, rx): (CompletionStreamSender, CompletionStreamReceiver) =
            tokio::sync::mpsc::channel(50);
        CompletionStreamingThread::spawn_poll_stream_for_tokens(response_stream, tx);
        CompletionReceiverHandler::from(rx)
    }

    #[cfg(feature = "long_term_memory")]
    pub fn vector_query_files(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
        match &self.context.memory {
            MemoryVariant::Long(mem) => {
                let query_vector = embed(query).expect("Failed to embed query");
                Some(File::get_from_embedding(query_vector.into(), &mem.pool))
            }
            _ => None,
        }
    }

    #[cfg(feature = "long_term_memory")]
    pub fn vector_query_chunks(&mut self, query: &str) -> Option<Vec<EmbeddedCoreStruct>> {
        match &self.context.memory {
            MemoryVariant::Long(mem) => {
                let query_vector = embed(query).expect("Failed to embed query");
                Some(FileChunk::get_from_embedding(
                    query_vector.into(),
                    &mem.pool,
                ))
            }
            _ => None,
        }
    }
}
