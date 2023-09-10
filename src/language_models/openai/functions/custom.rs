use super::config::{Function, FunctionArgument, Parameters, Property};

#[derive(Debug)]
pub struct CustomFunction {
    name: String,
    description: String,
    args: Vec<FunctionArgument>,
}

#[derive(Default, Debug)]
pub struct CustomFunctionBuilder {
    name: Option<String>,
    description: Option<String>,
    arguments: Option<Vec<FunctionArgument>>,
}

impl CustomFunction {
    pub fn build_from(name: &str) -> CustomFunctionBuilder {
        let mut builder = CustomFunctionBuilder::default();
        builder.name(name);
        builder
    }

    pub fn function(&self) -> Function {
        let mut builder = Parameters::build();
        let builder = builder.type_declaration("object");

        for property in self.args.iter() {
            builder.add_property(property);
        }
        let parameters = builder.finished();

        let function = Function::build_from(&self.name)
            .description(&self.description)
            .parameters(parameters)
            .finished();

        function
    }
}

impl CustomFunctionBuilder {
    fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn add_property(&mut self, property: Property, required: bool) -> &mut Self {
        let arg_to_push = match required {
            true => FunctionArgument::Required(property),
            false => FunctionArgument::Optional(property),
        };
        match &mut self.arguments {
            Some(args) => args.push(arg_to_push),
            None => self.arguments = Some(vec![arg_to_push]),
        }
        self
    }

    pub fn finished(&self) -> CustomFunction {
        let name = self.name.to_owned().unwrap();
        let description = self.description.to_owned().unwrap();
        let args = self.arguments.to_owned().unwrap();
        CustomFunction {
            name,
            description,
            args,
        }
    }
}
// Made to reflect OpenAi's function example from:
// https://openai.com/blog/function-calling-and-other-api-updates
//
// impl Into<Function> for FnEnum {
//     #[tracing::instrument(name = "Get function struct from FnEnum")]
//     fn into(self) -> Function {
//         match self {
//             FnEnum::GetCommands => {
//                 let properties = [Property::new(
//                     "commands",
//                     "array",
//                     "a list of terminal commands to be executed",
//                     &[
//                         ("type", "string"),
//                         ("description", "a terminal command string"),
//                     ],
//                 )];
//                 let parameters = Parameters::new("object", &properties, &["commands"]);
//                 Function::new(
//                     "get_commands",
//                     "get a list of terminal commands to run on mac os",
//                     parameters,
//                 )
//             }
//
//             FnEnum::RelevantFiles => {
//                 let properties = [Property::new(
//                     "files",
//                     "array",
//                     "a list of files to process",
//                     &[("type", "string"), ("description", "a file path string")],
//                 )];
//                 let parameters = Parameters::new("object", &properties, &["files"]);
//                 Function::new(
//                     "relevent_files",
//                     "Given a message, get a list of relavent files. Be extremely thorough. If a file is included in the message, it should be included in the resulting array.",
//                     parameters,
//                 )
//             }
//
//             FnEnum::SplitErrorMessage => {
//                 let properties = [Property::new(
//                     "errors",
//                     "array",
//                     "a list of errors to process",
//                     &[("type", "string"), ("description", "an error message")],
//                 )];
//                 let parameters = Parameters::new("object", &properties, &["erros"]);
//                 Function::new(
//                     "split_error_messaage",
//                     "Given stdout from an error message, split the error message by file.",
//                     parameters,
//                 )
//             }
//
//             FnEnum::ProblemSolveTasklist => {
//                 let properties = [Property::new(
//                     "tasks",
//                     "array",
//                     "a list of fine grained tasks, which by at the end, the problem will be solved",
//                     &[("type", "string"), ("description", "A simple task")],
//                 )];
//                 let parameters = Parameters::new("object", &properties, &["tasks"]);
//                 Function::new(
//                     "problem_solve_tasklist",
//                     "Given a description of a coding problem, it's associated error message, and relevant files, create a list of simple tasks to solve the problem.",
//                     parameters,
//                 )
//             }
//
//             FnEnum::ExecuteGenerateRead => {
//                 let properties = [Property::new(
//                     "classifications",
//                     "array",
//                     "a list of fine classifications for each of the given tasks",
//                     &[
//                         ("type", "string"),
//                         ("description", "Either Execute, Generate, Read"),
//                     ],
//                 )];
//                 let parameters = Parameters::new("object", &properties, &["classifications"]);
//                 Function::new(
//                     "execute_generate_read",
//                     "Given the list of tasks classify whether each is a request to generate code, execute a file or shell command, or simply read currently unavailable information.",
//                     parameters,
//                 )
//             }
//
//             #[cfg(feature = "meal_planner")]
//             FnEnum::ParseRecipeForIngredients => {
//                 let properties = [Property::new(
//                     "food_group",
//                     "enum",
//                     r#"A variant which can be one of any of these 15 options:
//                          - Proteins
//                          - GrainsAndCereals
//                          - Fruits
//                          - Vegetables
//                          - DairyAndDairyAlternative
//                          - FatsAndOils
//                          - Sweeteners
//                          - HerbsAndSpices
//                          - NutsAndSeeds
//                          - BakingIngredients
//                          - CondimentsAndSauces
//                          - Beverages
//                          - CannedAndPackagedFoods
//                          - MeatAlternatives
//                          - Seafood
//                         "#,
//                     &[("type", "enum"), ("description", "A FoodGroup variant")],
//                 )];
//                 let parameters = Parameters::new("object", &properties, &["classifications"]);
//                 Function::new(
//                     "parse_recipe",
//                     "Parse a given recipe in plaintext to into a list of ingredients",
//                     parameters,
//                 )
//             }
//         }
//     }
// }
