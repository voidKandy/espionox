use crate::{
    agent::config::{
        context::Context,
        memory::{
            LoadedMemory::{Cache, LongTerm},
            Memory,
        },
    },
    core::{file_interface::File, io::Io},
};
use crate::{
    database::init::DbPool,
    language_models::openai::{
        functions::{config::Function, enums::FnEnum},
        gpt::Gpt,
    },
};
use std::error::Error;
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

    pub async fn summarize(&mut self, content: &str) -> String {
        let save_mem = self.context.memory.clone();
        self.context.switch_mem(Memory::Forget);
        let summarize_prompt = format!("Summarize the core function code to the best of your ability. Be as succinct as possible. Content: {}", content);
        let response = self.prompt(&summarize_prompt);
        self.context.switch_mem(save_mem);
        response
    }

    pub async fn command(&mut self, command: &str) {
        self.io.push(Io::new(command))
    }
    //
    // async fn get_fix(&mut self) -> Result<String, Box<dyn Error>> {
    //     println!("_-Getting-Help-_");
    //     let content = match self.io.last() {
    //         Some(Io(i, o)) => {
    //             format!(
    //                 "This command was run: [{}]\nWhich resulted in this error: [{}]",
    //                 i, o
    //             )
    //         }
    //         None => {
    //             return Err("No io".into());
    //         }
    //     };
    //     self.context.push_to_buffer("user", &content);
    //
    //     let relevant_paths = self
    //         .function_prompt(&FnEnum::RelevantFiles.to_function())
    //         .await
    //         .unwrap();
    //
    //     let mut relevant_files = relevant_paths
    //         .into_iter()
    //         .map(|f| File::build(&f))
    //         .collect::<Vec<File>>();
    //     for file in relevant_files.iter_mut() {
    //         file.summary = self.summarize(&file.content()).await;
    //     }
    //     // relevant_files.make_relevant(&mut self.context);
    //     relevant_files.iter().for_each(|f| {
    //         let content = format!(
    //             "FilePath: {}\nSummary of the file's contents: {}",
    //             f.filepath.display().to_string(),
    //             f.summary,
    //         );
    //         self.context.push_to_buffer("user", &content);
    //     });
    //
    //     self.context.push_to_buffer("user", "Given the files and the error message, clearly express what the most urgent problem is and which single file it is in. If you know how to solve the problem, explain how it can be fixed.");
    //     let help = match self.prompt().await {
    //         Ok(response) => response,
    //         Err(err) => panic!("Error broke completion: {:?}", err),
    //     };
    //
    //     //// May need to spawn a thread here, Meaning we'll need multiple short term threads
    //     ///available. I dont think a new context should be initialized for this... We need a 'side
    //     ///job' implemtation
    //     let mut mem = Memory::Forget.load();
    //     mem.push_to_buffer("system", &help);
    //     let relevant_paths = self
    //         .function_prompt(&FnEnum::RelevantFiles.to_function())
    //         .await
    //         .unwrap();
    //
    //     mem = Memory::Temporary.init();
    //     let content = "You are a fixer agent. You will be given the summation of a programming error and it's solution. You will also be given the contents of the file that caused the error as it is. Your job is to recreate the file exactly as it is, except for the change that must be made to fix the error. Output should be only the recreated file content with the fix implemented.";
    //     mem.append_to_messages("system", content);
    //
    //     let file = File::build(&relevant_paths[0]);(
    //     let content = &format!(
    //         "Here is the file: {}\nHere is the error and it's proposed solution: {}",
    //         file.content(),
    //         &help
    //     );
    //     mem.append_to_messages("user", content);
    //
    //     let fix = match self.prompt().await {
    //         Ok(response) => response,
    //         Err(err) => panic!("Error broke completion: {:?}", err),
    //     };
    //     Ok(fix)
    // }
    //

    pub fn prompt(&mut self, input: &str) -> String {
        let (tx, rx) = mpsc::channel();
        let gpt = self.gpt.clone();
        self.context.push_to_buffer("assistant", &input);
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

        let result = rx
            .recv()
            .unwrap()
            .parse_response()
            .expect("Failed to parse completion response");
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
