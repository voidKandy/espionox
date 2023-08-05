use crate::language_models::openai::gpt::StreamResponse;
use crate::language_models::openai::{functions::config::Function, gpt::Gpt};
use crate::{
    agent::config::{
        context::Context,
        memory::{
            LoadedMemory::{Cache, LongTerm},
            Memory,
        },
    },
    core::io::Io,
};
use serde_json::Value;
use std::sync::mpsc;
use std::thread;
use tokio::runtime::Runtime;

pub struct Agent {
    pub gpt: Gpt,
    pub context: Context,
    pub io: Vec<Io>,
}

impl Agent {
    pub fn cache() -> Agent {
        let init_prompt ="You are Consoxide, a smart terminal. You help users with their programming experience by providing all kinds of services.".to_string();
        Agent {
            gpt: Gpt::init(&init_prompt),
            context: Context::build(Memory::Remember(Cache)),
            io: Vec::new(),
        }
    }

    pub fn long_term(name: &str) -> Agent {
        let init_prompt ="You are Consoxide, a smart terminal. You help users with their programming experience by providing all kinds of services.".to_string();
        Agent {
            gpt: Gpt::init(&init_prompt),
            context: Context::build(Memory::Remember(LongTerm(name.to_string()))),
            io: Vec::new(),
        }
    }

    pub fn forget() -> Agent {
        let init_prompt ="You are Consoxide, a smart terminal. You help users with their programming experience by providing all kinds of services.".to_string();
        Agent {
            gpt: Gpt::init(&init_prompt),
            context: Context::build(Memory::Forget),
            io: Vec::new(),
        }
    }

    pub fn save_buffer(&self) {
        self.context.memory.save(self.context.buffer.clone());
    }

    pub fn remember(&mut self, o: impl super::memorable::Memorable) {
        let mem = o.memorize();
        self.context.push_to_buffer("user", &mem);
        self.save_buffer();
        // todo!("Match to handle cache and long term");
    }

    fn switch_mem(&mut self, memory: Memory) {
        self.save_buffer();
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

    pub async fn command(&mut self, command: &str) {
        self.io.push(Io::new(command))
    }

    pub fn prompt(&mut self, input: &str) -> String {
        self.context.push_to_buffer("assistant", &input);

        let (tx, rx) = mpsc::channel();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async move {
                gpt.completion(&buffer)
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
            .parse_response()
            .expect("Failed to parse completion response");

        self.context.push_to_buffer("assistant", &result);
        result
    }

    pub fn stream_prompt(&mut self, input: &str) -> String {
        self.context.push_to_buffer("assistant", &input);

        let (tx, rx) = mpsc::channel();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async move {
                let mut response = gpt
                    .stream_completion(&buffer)
                    .await
                    .expect("Failed to get completion.");
                let mut body = Vec::new();
                while let Some(chunk) = response.chunk().await.unwrap() {
                    let res_str = String::from_utf8_lossy(&chunk).to_string();

                    println!("FROM WHILE LET LOOP: {} ...END", res_str);
                    if res_str == "data: [Done]" {
                        println!("DONE");
                        break;
                    }

                    let res_json = &res_str[res_str.find('{').unwrap()..].trim();

                    // if res_str.split(' ').nth(0) == Some("next:") {
                    //     let res_str = format!("{{{}}}", res_str);
                    // }
                    let data: Value =
                        serde_json::from_str(&res_json).expect("Failed to parse JSON");

                    // Access the "choices" field
                    let choices = data["choices"].as_array().expect("Expected choices array");
                    // let gpt_res: serde_json::Value =
                    //     serde_json::from_str::<serde_json::Value>(&res_str).unwrap();
                    body.push(res_str);
                }
                body.to_owned()
            });
            tx.send(result).unwrap();
        })
        .join()
        .expect("Failed to join thread");
        // let result = rx.recv().unwrap();
        let result = rx
            .recv()
            .unwrap()
            .into_iter()
            .map(|cow| cow.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        // .parse_response()

        self.context.push_to_buffer("assistant", &result);
        result
    }

    pub fn function_prompt(&mut self, function: Function) -> Vec<String> {
        let (tx, rx) = mpsc::channel();
        let gpt = self.gpt.clone();
        let buffer = self.context.buffer.clone();
        let function_name = &function.perameters.properties[0].name.clone();

        thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async move {
                gpt.function_completion(&buffer, &function)
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
            .parse_fn_response(&function_name)
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
