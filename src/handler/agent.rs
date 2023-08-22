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

#[derive(Debug, Default)]
pub struct Agent {
    pub context: Context,
    gpt: Gpt,
    settings: AgentSettings,
}

impl Agent {
    pub fn build(settings: AgentSettings) -> Result<Agent> {
        Ok(Agent::initialize(
            &mut Agent {
                gpt: Gpt::init(),
                context: Context::build(Memory::default()),
                settings,
            },
            move |agent| agent.build_from_settings(),
        ))
    }

    pub fn build_from_settings(&mut self) -> &mut Agent {
        match &self.settings.threadname {
            Some(name) => {
                self.switch_mem(Memory::LongTerm(name.to_string()));
            }
            None => {}
        }

        if self.context.buffer.len() == 0 {
            for p in self.settings.init_prompt.as_ref() {
                self.context.push_to_buffer(p.role(), p.content());
            }
        }

        self
    }

    pub fn initialize<F>(agent: &mut Agent, mut func: F) -> Agent
    where
        F: FnMut(&mut Agent) -> &mut Agent,
    {
        std::mem::take(func(agent))
    }

    pub fn remember(&mut self, o: impl super::super::core::Memorable) {
        let mem = o.memorize();
        self.context.push_to_buffer("user", &mem);
        self.context.save_buffer();
        // todo!("Match to handle cache and long term");
    }

    pub fn switch_mem(&mut self, memory: Memory) {
        self.context.save_buffer();
        self.context = Context::build(memory);
    }

    pub async fn summarize(&mut self, content: &str) -> String {
        let save_mem = self.context.memory.clone();
        self.switch_mem(Memory::Forget);
        let summarize_prompt = format!("Summarize the core function code to the best of your ability. Be as succinct as possible. Content: {}", content);
        let response = self.prompt(&summarize_prompt);
        self.switch_mem(save_mem);
        response
    }

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
