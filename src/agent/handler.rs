use super::context::{
    config::{Context, Contextual, Memory},
    walk::File,
};
use super::functions::config::Function;
use super::functions::enums::FnEnum;
use crate::api::gpt::Gpt;
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

    pub async fn summarize(&mut self, content: &str) -> String {
        self.context.switch(Memory::Temporary);
        let summarize_prompt = format!("Summarize the given code to the best of your ability. Be as succinct as possible while also being as thorough as possible. Content: {}", content);
        self.context.append_to_messages("system", &summarize_prompt);
        let response = self.prompt().await.unwrap();
        self.context.switch(Memory::ShortTerm);
        response
    }

    pub async fn monitor_user(&mut self) {
        // loop {
        self.context.session.watch();
        if self.context.session.is_problematic() {
            println!("[PROBLEM DETECTED]");
            self.offer_help().await.unwrap();
        };
        // }
    }

    async fn offer_help(&mut self) -> Result<(), Box<dyn Error>> {
        let content = match self.context.session.contents.iter().last() {
            Some((last_in, last_out)) => {
                format!(
                    "This command was run: {}, Which resulted in this error: {}",
                    last_in, last_out
                )
            }
            None => {
                return Err("No Context".into());
            }
        };
        self.context.append_to_messages("system", &content);
        let relevant_paths = self
            .function_prompt(&FnEnum::RelevantFiles.to_function())
            .await
            .unwrap();

        self.context
            .session
            .to_out(&format!("Relavent Files: [{relevant_paths:?}]",));

        let relevant_files = relevant_paths
            .into_iter()
            .map(|f| File::build(&f))
            .collect::<Vec<File>>();
        // for file in relevant_files.iter_mut() {
        //     file.summary = self.summarize(&file.content()).await;
        // }
        relevant_files.make_relevant(&mut self.context);

        self.context.append_to_messages("system", "Given the files and the error message, clearly express what the most urgent problem is. If you know how to solve the problem, show a code snippet of how to solve it.");
        // println!("{:?}", self.context.messages);
        let help = self.prompt().await.unwrap();
        self.context.session.to_out(&help);
        println!("{}", help);
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
