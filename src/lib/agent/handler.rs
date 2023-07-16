use super::functions::config::Function;
use super::functions::enums::FnEnum;
use crate::lib::io::tmux_session::InSession;
use crate::lib::models::gpt::Gpt;
use crate::lib::{
    agent::config::{
        context::{Context, Contextual},
        memory::Memory,
    },
    io::walk::File,
};
use std::error::Error;
use std::fmt::format;

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
        let content = match self.context.session.io.iter().last() {
            Some((i, o)) => {
                format!(
                    "This command was run: [{}]\nWhich resulted in this error: {}",
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

        self.context.append_to_messages("user", "Given the files and the error message, clearly express what the most urgent problem is. If you know how to solve the problem, show a code snippet of how to solve it.");
        let help = match self.prompt().await {
            Ok(response) => response,
            Err(err) => panic!("Error broke completion: {:?}", err),
        };
        self.context.session.to_out(&format!("{}\n", &help));
        Ok(())
    }

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
