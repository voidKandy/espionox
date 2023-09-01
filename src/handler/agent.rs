use super::AgentSettings;
use crate::context::{memory::Memory, Context};
use crate::language_models::openai::{
    functions::config::Function,
    gpt::{Gpt, StreamResponse},
};
use anyhow::Result;
use bytes::Bytes;
use futures::Stream;
use futures_util::StreamExt;
use std::{sync::mpsc, thread};
use tokio::runtime::Runtime;

#[derive(Debug)]
pub struct Agent {
    pub context: Context,
    gpt: Gpt,
    // settings: AgentSettings,
}

impl Default for Agent {
    fn default() -> Self {
        Agent::build(AgentSettings::default()).expect("Failed to build default agent")
    }
}

impl Agent {
    pub fn build(settings: AgentSettings) -> Result<Agent> {
        let gpt = Gpt::init();
        let context = match &settings.memory_override {
            Some(memory) => Context::build(memory.clone()),
            None => Context::build(Memory::default()),
        };
        Ok(Agent {
            gpt,
            context,
            // settings,
        })
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
        let buffer = self.context.buffer_as_string();
        let current_mem = match &self.context.memory {
            Memory::Forget => "Forget".to_string(),
            Memory::ShortTerm => "ShortTerm".to_string(),
            Memory::LongTerm(thread) => {
                format!("LongTerm Thread: {}", thread.clone())
            }
        };
        format!("In {current_mem}\n\nBuffer:\n{buffer}")
    }

    pub fn format_to_buffer(&mut self, o: impl super::integrations::BufferDisplay) {
        let mem = o.buffer_display();
        self.context.push_to_buffer("user", &mem);
        // todo!("Match to handle cache and long term");
    }

    pub fn switch_mem(&mut self, memory: Memory) {
        self.context.save_buffer();
        self.context = Context::build(memory);
    }

    // pub fn get_summary(&mut self, content: &str) -> String {
    //     let save_mem = self.context.memory.clone();
    //     self.switch_mem(Memory::Forget);
    //     let summarizer_prompt =
    //             r#"You are a code summarization Ai, you will be given a chunk of code to summarize
    //             - Mistakes erode user's trust, so be as accurate and thorough as possible
    //             - Be highly organized
    //             - think through your response step by step, your summary should be succinct but accurate"#.to_string();
    //     self.context.push_to_buffer("system", &summarizer_prompt);
    //     let response = self.prompt(&content);
    //     self.switch_mem(save_mem);
    //     response
    // }

    pub async fn async_prompt(&mut self, input: &str) -> String {
        self.context.push_to_buffer("user", &input);

        let gpt = self.gpt.clone();
        let result = gpt
            .completion(&self.context.buffer.clone().into())
            .await
            .expect("Failed to get completion.")
            .parse()
            .expect("Failed to parse completion response");

        self.context.push_to_buffer("assistant", &result);
        result
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
        // let result = rx.recv().unwrap();
        let result = rx
            .recv()
            .unwrap()
            .parse()
            .expect("Failed to parse completion response");

        self.context.push_to_buffer("assistant", &result);
        result
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

    #[tracing::instrument]
    pub fn stream_prompt(
        &mut self,
        input: &str,
    ) -> tokio::sync::mpsc::Receiver<Result<String, anyhow::Error>> {
        self.context.push_to_buffer("assistant", &input);
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        let mut response = thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                gpt.stream_completion(&buffer.into())
                    .await
                    .expect("Failed to get completion.")
            })
        })
        .join()
        .unwrap();

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        tokio::spawn(async move {
            while let Ok(Some(token)) = Self::poll_stream_for_token(&mut response).await {
                tx.send(Ok(token)).await.unwrap();
            }

            // tokio::time::sleep(Duration::from_millis(200)).await;
        });
        rx
        // self.context.push_to_buffer("assistant", &result);
        // Ok(handle)
    }

    #[tracing::instrument(name = "Function prompt GPT API for response")]
    pub fn function_prompt(&mut self, function: Function) -> Vec<String> {
        let (tx, rx) = mpsc::channel();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        let function_name = &function.perameters.properties[0].name.clone();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async move {
                gpt.function_completion(&buffer.into(), &function)
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
            .parse_fn(&function_name)
            .expect("Failed to parse completion response")
            .clone()
            .into_iter()
            .map(|c| {
                self.context.push_to_buffer("assistant", &c);
                c
            })
            .collect();

        result
    }
}
