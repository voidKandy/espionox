use crate::language_models::openai::{
    functions::{config::Function, enums::FnEnum},
    gpt::Gpt,
};
use crate::{
    agent::config::{context::Context, memory::Memory},
    core::{file_interface::File, io::Io},
};
use std::error::Error;

pub struct Agent {
    pub gpt: Gpt,
    pub context: Context,
    pub io: Vec<Io>,
}

impl Agent {
    pub fn new(memory: Memory) -> Agent {
        let init_prompt ="You are Consoxide, a smart terminal. You help users with their programming experience by providing all kinds of services.".to_string();
        Agent {
            gpt: Gpt::init(&init_prompt),
            context: Context::build(&memory),
            io: Vec::new(),
        }
    }

    pub async fn summarize(&mut self, content: &str) -> String {
        let save_mem = self.context.memory.clone();
        self.context.switch_mem(Memory::Forget);
        let summarize_prompt = format!("Summarize the core function code to the best of your ability. Be as succinct as possible. Content: {}", content);
        self.context.push_to_buffer("system", &summarize_prompt);
        let response = self.prompt().await.unwrap();
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
    //     let file = File::build(&relevant_paths[0]);
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

    pub async fn prompt(&mut self) -> Result<String, Box<dyn Error>> {
        match self
            .gpt
            .completion(&self.context.buffer)
            .await?
            .parse_response()
        {
            Ok(content) => {
                self.context.push_to_buffer("assistant", &content);
                Ok(content)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn function_prompt(
        &mut self,
        function: &Function,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        match self
            .gpt
            .function_completion(&self.context.buffer, &function)
            .await?
            .parse_fn_response(&function.perameters.properties[0].name)
        {
            Ok(content) => {
                content.clone().into_iter().for_each(|c| {
                    self.context.push_to_buffer("assistant", &c);
                });
                Ok(content)
            }
            Err(err) => Err(err),
        }
    }
}
