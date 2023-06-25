#[cfg(test)]
use crate::agent::functions::Function;

#[test]
fn test_function_build() {
    let function = Function::new(
        "get_commands",
        "get a list of terminal commands to run on mac os",
        r#"
        {
            "type": "object",
            "properties": {
                "commands": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "description": "a terminal command string"
                    },
                    "description": "list of terminal commands to be executed"
                }
            },
            "required": ["commands"]
        }
        "#,
    );

    let expected_result = r#"
        {
            "functions": [
                {
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
                                "description": "list of terminal commands to be executed"
                            }
                        },
                        "required": ["commands"]
                    }
                }
            ],
            "function_call": {
                "name": "get_commands"
            }
        }
        "#;
    assert_eq!(
        function.build().replace("\n", ""),
        expected_result.replace("\n", "")
    );
}
