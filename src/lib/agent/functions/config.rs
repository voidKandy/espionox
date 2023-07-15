pub struct Function {
    pub name: String,
    pub description: String,
    pub perameters: Perameters,
}
pub struct Perameters {
    pub type_dec: String,
    pub properties: Box<Vec<Property>>,
    pub required: Vec<String>,
}
#[derive(Clone)]
pub struct Property {
    pub name: String,
    pub return_value: String,
    pub items: Vec<(String, String)>,
    pub description: String,
}

impl Function {
    pub fn new(name: &str, description: &str, perameters: Perameters) -> Function {
        Function {
            name: name.to_string(),
            description: description.to_string(),
            perameters,
        }
    }
    pub fn render(&self) -> String {
        let name = format!("\"name\": \"{}\"", self.name);
        let description = format!("\"description\": \"{}\"", self.description);
        format!(
            "{{ {}, {}, {} }}",
            name,
            description,
            self.perameters.render(),
        )
    }
}

impl Perameters {
    pub fn new(type_dec: &str, properties: &[Property], required: &[&str]) -> Perameters {
        let type_dec = type_dec.to_string();
        let props: Vec<Property> = properties.iter().cloned().collect();
        let required = required.into_iter().map(|s| s.to_string()).collect();
        Perameters {
            type_dec,
            properties: Box::new(props),
            required,
        }
    }
    pub fn render(&self) -> String {
        let type_dec = format!("\"type\": \"{}\"", self.type_dec);
        let required = format!("\"required\": [\"{}\"]", self.required.join(", "));
        let properties = self
            .properties
            .iter()
            .map(|p| p.render())
            .collect::<Vec<String>>()
            .join(",\n");
        format!(
            " \"parameters\": {{
                {},
                \"properties\": {{
                {}
                }},
                {}
            }}",
            type_dec, properties, required
        )
    }
}

impl Property {
    pub fn new(
        name: &str,
        return_value: &str,
        description: &str,
        items: &[(&str, &str)],
    ) -> Property {
        let name = name.to_string();
        let return_value = return_value.to_string();
        let description = description.to_string();
        let items = items
            .into_iter()
            .map(|&(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        Property {
            name,
            return_value,
            items,
            description,
        }
    }
    fn render_items(&self) -> String {
        let items: &Vec<String> = &self
            .items
            .clone()
            .into_iter()
            .map(|(k, v)| format!("\"{}\": \"{}\"", k, v))
            .collect::<Vec<String>>();
        format!("\"items\": {{{}}}", items.join(", "))
    }
    pub fn render(&self) -> String {
        format!(
            "\"{}\": {{\"type\": \"{}\", {}, \"description\": \"{}\"}}",
            self.name,
            self.return_value,
            self.render_items(),
            self.description
        )
    }
}
