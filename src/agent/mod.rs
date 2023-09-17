pub mod settings;
pub mod spo_agents;

pub use settings::AgentSettings;

use crate::{
    configuration::ConfigEnv,
    context::{
        integrations::{
            core::BufferDisplay,
            database::{Embedded, EmbeddedCoreStruct},
        },
        Context, Memory,
    },
    core::{File, FileChunk},
    language_models::{
        embed,
        openai::{
            functions::CustomFunction,
            gpt::{Gpt, StreamResponse},
        },
    },
};
use bytes::Bytes;
use futures::Stream;
use futures_util::StreamExt;
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
            None => Context::build(Memory::default(), env),
        };

        match context.memory {
            Memory::Forget => context.buffer = settings.init_prompt,
            _ => {
                if context.buffer.len() == 0 {
                    context.buffer = settings.init_prompt;
                }
            }
        }

        Ok(Agent { gpt, context })
    }

    pub fn vector_query_files(&mut self, query: &str) -> Vec<EmbeddedCoreStruct> {
        let query_vector = embed(query).expect("Failed to embed query");
        File::get_from_embedding(query_vector.into(), self.context.pool())
    }

    pub fn vector_query_chunks(&mut self, query: &str) -> Vec<EmbeddedCoreStruct> {
        let query_vector = embed(query).expect("Failed to embed query");
        FileChunk::get_from_embedding(query_vector.into(), self.context.pool())
    }

    pub fn build_with<F>(agent: &mut Agent, mut func: F) -> Agent
    where
        F: FnMut(&mut Agent) -> &mut Agent,
    {
        std::mem::take(func(agent))
    }

    pub fn do_with<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut Self),
    {
        std::mem::take(&mut func(self))
    }

    pub fn info_display_string(&self) -> String {
        match &self.context.memory {
            Memory::Forget => "Forget".to_string(),
            Memory::ShortTerm => "ShortTerm".to_string(),
            Memory::LongTerm(thread) => {
                format!("{}", thread.clone())
            }
        }
    }

    pub fn format_to_buffer(&mut self, o: impl BufferDisplay) {
        let mem = o.buffer_display();
        self.context.push_to_buffer("user", &mem);
    }

    pub fn switch_mem(&mut self, memory: Memory) {
        self.context.save_buffer();
        self.context = Context::build(memory, self.context.env.to_owned());
    }

    #[tracing::instrument(name = "Prompt GPT API for response")]
    pub fn prompt(&mut self, input: &str) -> String {
        self.context.push_to_buffer("user", &input);

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

        self.context.push_to_buffer("assistant", &result);
        result
    }

    #[tracing::instrument(name = "Function prompt GPT API for response" skip(input, custom_function))]
    pub fn function_prompt(&mut self, custom_function: CustomFunction, input: &str) -> Value {
        self.context.push_to_buffer("user", &input);
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
    pub async fn stream_prompt(
        &mut self,
        input: &str,
    ) -> tokio::sync::mpsc::Receiver<Result<String, anyhow::Error>> {
        self.context.push_to_buffer("assistant", &input);
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        let mut response = gpt
            .stream_completion(&buffer.into())
            .await
            .expect("Failed to get completion.");

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            while let Ok(Some(token)) = Self::poll_stream_for_token(&mut response).await {
                tracing::info!("Token got: {}", token);
                tx.send(Ok(token)).await.unwrap();
            }
        });
        rx
    }

    async fn poll_stream_for_token(
        mut response: impl Stream<Item = Result<Bytes, reqwest::Error>> + std::marker::Unpin,
    ) -> anyhow::Result<Option<String>> {
        if let Some(Ok(chunk)) = response.next().await {
            match StreamResponse::from_byte_chunk(chunk).await {
                Ok(stream_res) => {
                    let parsed_res = stream_res.parse().unwrap();
                    Ok(Some(parsed_res))
                }
                Err(err) => {
                    Err(anyhow::anyhow!("Problem getting stream response: {:?}", err).into())
                }
            }
        } else {
            Ok(None)
        }
    }
}
