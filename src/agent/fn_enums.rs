use crate::agent::fn_render::{Function, Perameters, Property};

#[derive(Clone)]
pub enum FnEnum {
    // GeneralDescription,
    GetCommands,
    RelevantFiles,
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
                    "get a list of relevant files from a code snippet",
                    perameters,
                )
            }
        }
    }
}
