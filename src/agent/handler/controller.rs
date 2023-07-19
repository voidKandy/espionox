use super::super::functions::config::Function;
use super::super::functions::enums::FnEnum;
use crate::io::tmux::pane::InSession;
use crate::language_models::gpt::Gpt;
use crate::{
    agent::{
        config::memory::Memory,
        handler::context::{Context, Contextual},
    },
    io::file_interface::File,
};
use std::error::Error;

pub struct AgentHandler {
    pub gpt: Gpt,
    pub context: Context,
}

impl AgentHandler {
    pub fn new(context: Memory) -> AgentHandler {
        let init_prompt ="You are Consoxide, a smart terminal. You help users with their programming experience by providing all kinds of services.".to_string();
        AgentHandler {
            gpt: Gpt::init(&init_prompt),
            context: context.init(),
        }
    }

    pub async fn true_relavent_files(&mut self) -> Vec<String> {
        let relavent_file_names = &mut self
            .function_prompt(&FnEnum::RelevantFiles.to_function())
            .await
            .expect("RelevantFiles failure");
        relavent_file_names.iter_mut().for_each(|n| {
            // let name = &n.rsplit("/").collect::<Vec<&str>>()[..2];
            // let true_prefix = File::get_prefix(&n);
            // format!("{}{}", &true_prefix, &n)
        });
        relavent_file_names.to_vec()
    }

    pub async fn summarize(&mut self, content: &str) -> String {
        self.context.switch(Memory::Temporary);
        let summarize_prompt = format!("Summarize the core function code to the best of your ability. Be as succinct as possible. Content: {}", content);
        self.context.append_to_messages("system", &summarize_prompt);
        let response = self.prompt().await.unwrap();
        self.context.switch(Memory::ShortTerm);
        response
    }

    pub async fn monitor_user(&mut self) {
        // loop {
        let (i, o) = self.context.session.watched_pane.cl_io();
        self.context.session.io.insert(i, o);
        self.offer_help().await.unwrap();
        // }
    }

    async fn offer_help(&mut self) -> Result<(), Box<dyn Error>> {
        println!("_-Getting-Help-_");
        let content = match self.context.session.io.iter().last() {
            Some((i, o)) => {
                format!(
                    "This command was run: [{}]\nWhich resulted in this error: [{}]",
                    i, o
                )
            }
            None => {
                return Err("No io".into());
            }
        };
        self.context.append_to_messages("user", &content);

        let relevant_paths = self
            .function_prompt(&FnEnum::RelevantFiles.to_function())
            .await
            .unwrap();
        self.context
            .session
            .to_out(&format!("Relavent Files: {relevant_paths:?}\n",));

        let mut relevant_files = relevant_paths
            .into_iter()
            .map(|f| File::build(&f))
            .collect::<Vec<File>>();
        for file in relevant_files.iter_mut() {
            file.summary = self.summarize(&file.content()).await;
        }
        // relevant_files.make_relevant(&mut self.context);
        relevant_files.iter().for_each(|f| {
            let content = format!(
                "FilePath: {}\nSummary of the file's contents: {}",
                f.filepath.display().to_string(),
                f.summary,
            );
            self.context.append_to_messages("user", &content);
        });

        self.context.append_to_messages("user", "Given the files and the error message, clearly express what the most urgent problem is and which single file it is in. If you know how to solve the problem, explain how it can be fixed.");
        let help = match self.prompt().await {
            Ok(response) => response,
            Err(err) => panic!("Error broke completion: {:?}", err),
        };
        self.context.session.to_out(&format!("{}\n", &help));

        // NOw fix

        let mut mem = Memory::Temporary.init();
        mem.append_to_messages("system", &help);
        let relevant_paths = self
            .function_prompt(&FnEnum::RelevantFiles.to_function())
            .await
            .unwrap();

        mem = Memory::Temporary.init();
        let content = "You are a fixer agent. You will be given the summation of a programming error and it's solution. You will also be given the contents of the file that caused the error as it is. Your job is to recreate the file exactly as it is, except for the change that must be made to fix the error. Output should be only the recreated file content with the fix implemented.";
        mem.append_to_messages("system", content);

        let file = File::build(&relevant_paths[0]);
        let content = &format!(
            "Here is the file: {}\nHere is the error and it's proposed solution: {}",
            file.content(),
            &help
        );
        mem.append_to_messages("user", content);

        let fix = match self.prompt().await {
            Ok(response) => response,
            Err(err) => panic!("Error broke completion: {:?}", err),
        };
        self.context.session.to_out(&format!("{}\n", &fix));
        Ok(())
    }

    pub fn fix(&mut self) {}

    pub async fn prompt(&mut self) -> Result<String, Box<dyn Error>> {
        match self
            .gpt
            .completion(&self.context.messages)
            .await?
            .parse_response()
        {
            Ok(content) => {
                self.context.append_to_messages("assistant", &content);
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
            .function_completion(&self.context.messages, &function)
            .await?
            .parse_fn_response(&function.perameters.properties[0].name)
        {
            Ok(content) => {
                content.clone().into_iter().for_each(|c| {
                    self.context.append_to_messages("assistant", &c);
                });
                Ok(content)
            }
            Err(err) => Err(err),
        }
    }
}
