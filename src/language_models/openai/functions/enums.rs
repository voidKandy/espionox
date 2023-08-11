use super::config::{Function, Perameters, Property};

#[derive(Clone)]
pub enum FnEnum {
    // GeneralDescription,
    GetCommands,
    RelevantFiles,
    SplitErrorMessage,
    ProblemSolveTasklist,
    ExecuteGenerateRead,
}

impl FnEnum {
    pub fn to_function(&self) -> Function {
        match self {
            // FnEnum::GeneralDescription => {
            //     let properties = [Property::new(
            // }
            FnEnum::GetCommands => {
                let properties = [Property::new(
                    "commands",
                    "array",
                    "a list of terminal commands to be executed",
                    &[
                        ("type", "string"),
                        ("description", "a terminal command string"),
                    ],
                )];
                let perameters = Perameters::new("object", &properties, &["commands"]);
                Function::new(
                    "get_commands",
                    "get a list of terminal commands to run on mac os",
                    perameters,
                )
            }

            FnEnum::RelevantFiles => {
                let properties = [Property::new(
                    "files",
                    "array",
                    "a list of files to process",
                    &[("type", "string"), ("description", "a file path string")],
                )];
                let perameters = Perameters::new("object", &properties, &["files"]);
                Function::new(
                    "relevent_files",
                    "Given a message, get a list of relavent files. Be extremely thorough. If a file is included in the message, it should be included in the resulting array.",
                    perameters,
                )
            }

            FnEnum::SplitErrorMessage => {
                let properties = [Property::new(
                    "errors",
                    "array",
                    "a list of errors to process",
                    &[("type", "string"), ("description", "an error message")],
                )];
                let perameters = Perameters::new("object", &properties, &["erros"]);
                Function::new(
                    "split_error_messaage",
                    "Given stdout from an error message, split the error message by file.",
                    perameters,
                )
            }

            FnEnum::ProblemSolveTasklist => {
                let properties = [Property::new(
                    "tasks",
                    "array",
                    "a list of fine grained tasks, which by at the end, the problem will be solved",
                    &[("type", "string"), ("description", "A simple task")],
                )];
                let perameters = Perameters::new("object", &properties, &["tasks"]);
                Function::new(
                    "problem_solve_tasklist",
                    "Given a description of a coding problem, it's associated error message, and relevant files, create a list of simple tasks to solve the problem.",
                    perameters,
                )
            }

            FnEnum::ExecuteGenerateRead => {
                let properties = [Property::new(
                    "classifications",
                    "array",
                    "a list of fine classifications for each of the given tasks",
                    &[
                        ("type", "string"),
                        ("description", "Either Execute, Generate, Read"),
                    ],
                )];
                let perameters = Perameters::new("object", &properties, &["classifications"]);
                Function::new(
                    "execute_generate_read",
                    "Given the list of tasks classify whether each is a request to generate code, execute a file or shell command, or simply read currently unavailable information.",
                    perameters,
                )
            }
        }
    }
}
