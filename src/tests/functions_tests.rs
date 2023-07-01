#[cfg(test)]
use crate::agent::{
    fn_enums::FnEnum,
    fn_render::{Function, Perameters, Property},
};
use serde_json::json;
use serde_json::Value;

#[test]
fn function_deserialization() {
    let function = FnEnum::GetCommands.get_function();
    println!("{}", function.render());

    let function_json = json!({
        "name": "get_commands",
        "description": "get a list of terminal commands to run on mac os",
        "parameters": {
            "type": "object",
            "properties": {
                "commands": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "description": "a terminal command string"
                    },
                    "description": "a list of terminal commands to be executed"
                }
            },
            "required": ["commands"]
        }
    });

    let function_render = r#"{ "name": "get_commands", "description": "get a list of terminal commands to run on mac os", "parameters": { "type": "object", "properties": { "commands": { "type": "array", "items": { "type": "string", "description": "a terminal command string" }, "description": "a list of terminal commands to be executed" } }, "required": ["commands"] } }"#;

    let functions_json: Result<Value, serde_json::Error> = serde_json::from_str(function_render);
    assert!(
        functions_json.is_ok(),
        "Deserialization failed: {:?}",
        functions_json
    );

    let json_value = functions_json.unwrap();
    assert_eq!(json_value["name"], "get_commands");
    assert_eq!(json_value["parameters"]["type"], "object");
    assert_eq!(json_value, function_json);

    let functions_json: Result<Value, serde_json::Error> = serde_json::from_str(&function.render());
    assert!(
        functions_json.is_ok(),
        "Deserialization failed: {:?}",
        functions_json
    );

    let json_value = functions_json.unwrap();
    assert_eq!(json_value["name"], "get_commands");
    assert_eq!(json_value["parameters"]["type"], "object");
    assert_eq!(json_value, function_json)
}
#[test]
fn test_function_render() {
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
    let function = Function::new(
        "get_commands",
        "get a list of terminal commands to run on mac os",
        perameters,
    );

    let expected_result = json!({
        "name": "get_commands",
        "description": "get a list of terminal commands to run on mac os",
        "parameters": {
            "type": "object",
            "properties": {
                "commands": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "description": "a terminal command string"
                    },
                    "description": "a list of terminal commands to be executed"
                }
            },
            "required": ["commands"]
        }
    });

    let rendered_json: Value = serde_json::from_str(&function.render()).unwrap();
    let expected_json = serde_json::to_string(&expected_result).unwrap();
    assert_eq!(rendered_json.to_string(), expected_json);
}
