use super::models::{Function, FunctionArgument, Parameters, Property};

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
