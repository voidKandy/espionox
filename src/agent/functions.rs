pub struct Function {
    name: String,
    description: String,
    perameters: String,
}

impl Function {
    pub fn new(name: &str, description: &str, perameters: &str) -> Function {
        Function {
            name: name.to_string(),
            description: description.to_string(),
            perameters: perameters.to_string(),
        }
    }
    pub fn build(&self) -> String {
        let name = format!("\"name\": \"{}\"", self.name);
        let description = format!("\"description\": \"{}\"", self.description);
        let parameters = format!("\"parameters\": {}", self.perameters);
        let call = format!("\"function_call\": {{\"name\": \"{}\"}}", self.name);
        format!(
            "{{\"functions\": [{{{},{},{}}}]{}}}",
            name, description, parameters, call
        )
    }
}
