use serde_json::{json, Value};

#[derive(Clone, Debug)]
pub struct Function {
    pub name: String,
    pub json: Value,
}

#[derive(Clone, Debug, Default)]
pub struct Parameters {
    json: Value,
}

#[derive(Clone, Debug)]
pub struct Property {
    pub name: String,
    json: Value,
}

#[derive(Debug, Clone)]
pub enum FunctionArgument {
    Required(Property),
    Optional(Property),
}

// ---- Builders ---- //

#[derive(Default, Debug, Clone)]
pub struct FunctionBuilder {
    name: String,
    description: String,
    parameters: Parameters,
}

#[derive(Default, Debug, Clone)]
pub struct ParametersBuilder {
    type_dec: String,
    properties: Vec<Property>,
    required_props: Vec<String>,
}

#[derive(Default, Debug, Clone)]
pub struct PropertyBuilder {
    name: String,
    return_type: String,
    info: Vec<PropertyInfo>,
}

#[derive(Default, Debug, Clone)]
pub struct PropertyInfo {
    fieldname: String,
    content: Value,
}

impl AsRef<Property> for FunctionArgument {
    fn as_ref(&self) -> &Property {
        match self {
            FunctionArgument::Required(v) => v,
            FunctionArgument::Optional(v) => v,
        }
    }
}

impl FunctionArgument {
    pub fn is_required(&self) -> bool {
        match self {
            FunctionArgument::Required(_) => true,
            _ => false,
        }
    }
}
impl Function {
    pub fn build_from(name: &str) -> FunctionBuilder {
        let mut builder = FunctionBuilder::default();
        builder.name(name);
        builder
    }
}

impl FunctionBuilder {
    fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn description(&mut self, description: &str) -> &mut Self {
        self.description = description.to_string();
        self
    }

    pub fn parameters(&mut self, parameters: Parameters) -> &mut Self {
        self.parameters = parameters;
        self
    }

    pub fn finished(&self) -> Function {
        let name = self.name.to_owned();
        let json = json!({
            "name": &name,
            "description": self.description,
            "parameters": self.parameters.json
        });
        Function { name, json }
    }
}

impl Parameters {
    pub fn build() -> ParametersBuilder {
        ParametersBuilder::default()
    }
}

impl ParametersBuilder {
    pub fn type_declaration(&mut self, type_dec: &str) -> &mut Self {
        self.type_dec = type_dec.to_string();
        self
    }

    pub fn add_property(&mut self, property: &FunctionArgument) -> &mut Self {
        match property {
            FunctionArgument::Required(arg) => self.required_props.push(arg.name.to_owned()),
            FunctionArgument::Optional(_) => {}
        }
        self.properties.push(property.as_ref().to_owned());
        self
    }

    pub fn finished(&self) -> Parameters {
        let mut json = json!({
            "type": self.type_dec,
            "properties": {},
            "required": []
        });

        for prop in self.properties.iter() {
            if let Some(obj) = json["properties"].as_object_mut() {
                obj.insert(prop.to_owned().name, prop.to_owned().json);
            }
        }

        for prop_name in self.required_props.iter() {
            if let Some(arr) = json["required"].as_array_mut() {
                arr.push(json!(prop_name));
            }
        }
        Parameters { json }
    }
}

impl Property {
    pub fn build_from(name: &str) -> PropertyBuilder {
        let mut builder = PropertyBuilder::default();
        builder.name(name);
        builder
    }
}

impl PropertyInfo {
    pub fn new(fieldname: &str, content: Value) -> Self {
        let fieldname = fieldname.to_string();
        Self { fieldname, content }
    }
}

impl PropertyBuilder {
    fn name(&mut self, name: &str) -> &mut Self {
        self.name = name.to_string();
        self
    }

    pub fn return_type(&mut self, return_type: &str) -> &mut Self {
        self.return_type = return_type.to_string();
        self
    }

    pub fn add_info(&mut self, info: PropertyInfo) -> &mut Self {
        self.info.push(info);
        self
    }

    pub fn finished(&self) -> Property {
        let name = self.name.to_owned();
        let mut json = json!({
                "type": self.return_type,
        });
        for i in self.info.iter() {
            if let Some(obj) = json.as_object_mut() {
                obj.insert(i.to_owned().fieldname, i.to_owned().content);
            }
        }
        Property { name, json }
    }
}
