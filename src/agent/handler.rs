use super::context::{
    config::{Context, Contextual},
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
    pub fn new() -> AgentHandler {
        let init_prompt ="You are Consoxide, a smart terminal. You help users with their programming experience by providing all kinds of services.".to_string();
        AgentHandler {
            gpt: Gpt::init(&init_prompt),
            context: Context::new("main", Some(&init_prompt)),
        }
    }

    pub async fn summarize(&mut self, content: &str) -> String {
        self.context.change_conversation("summarize");
        let summarize_prompt = format!("Summarize the given code to the best of your ability. Be as succinct as possible while also being as thorough as possible. Content: {}", content);
        self.context.append_to_messages("system", &summarize_prompt);
        let summary = self.prompt().await.unwrap();
        self.context.drop_conversation();
        summary
    }

    pub async fn monitor_user(&mut self) {
        // loop {
        self.context.pane.watch();
        if self.context.pane.is_problematic() {
            println!("[PROBLEM DETECTED]");
            self.offer_help().await.unwrap();
        };
        // }
    }

    async fn offer_help(&mut self) -> Result<(), Box<dyn Error>> {
        let content = match self.context.pane.contents.iter().last() {
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
        self.context.change_conversation("ProblemSolving");
        self.context.append_to_messages("system", &content);
        let relevant_files = self
            .function_prompt(&FnEnum::RelevantFiles.to_function())
            .await
            .unwrap();

        println!("{relevant_files:?}");
        let mut relevant_files = relevant_files
            .into_iter()
            .map(|f| File::build(&f))
            .collect::<Vec<File>>();
        for file in relevant_files.iter_mut() {
            file.summary = self.summarize(&file.content).await;
        }
        relevant_files.make_relevant(&mut self.context);

        self.context.append_to_messages("system", "Given the files and the error message, clearly express what the most urgent problem is. If you know how to solve the problem, show a code snippet of how to solve it.");
        println!("{}", self.prompt().await.unwrap());
        Ok(())
    }

    // async fn handle_problem(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
    //     match self.context.pane.contents.iter().last() {
    //         Some((last_in, last_out)) => {
    //             let content = format!(
    //                 "This command was run: {}, Which resulted in this error: {}",
    //                 last_in, last_out
    //             );
    //             self.context.append_to_messages("system", &content);
    //             let relevant_files = self
    //                 .function_prompt(&FnEnum::RelevantFiles.to_function())
    //                 .await
    //                 .unwrap();
    //
    //             println!("{relevant_files:?}");
    //             relevant_files
    //                 .into_iter()
    //                 .map(|f| File::build(&f))
    //                 .collect::<Vec<File>>()
    //                 .make_relevant(&mut self.context);
    //         }
    //         None => {
    //             return Err("No context".into());
    //         }
    //     }
    //     self.context.append_to_messages("system", "Given the files and the given error message, clearly express what the most urgent problem and and what files it pertains to.");
    //     println!("{}", self.prompt().await.unwrap());
    //     let tasks = self
    //         .function_prompt(&FnEnum::ProblemSolveTasklist.to_function())
    //         .await
    //         .unwrap();
    //     self.function_prompt(&FnEnum::ExecuteGenerateRead.to_function())
    //         .await
    // }

    pub async fn prompt(&mut self) -> Result<String, Box<dyn Error>> {
        match self
            .gpt
            .completion(&self.context.current_messages())
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
            .function_completion(&self.context.current_messages(), &function)
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
