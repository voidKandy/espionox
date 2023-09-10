use super::{agent::weather_test_function, helpers::init_test};
use consoxide::language_models::openai::functions::Function;
use serde_json::json;

#[test]
fn test_function_render() {
    init_test();
    let function: Function = weather_test_function().function();
    let expected_result = json!({
         "name": "get_current_weather",
              "description": "Get the current weather in a given location",
              "parameters": {
                "type": "object",
                "properties": {
                  "location": {
                    "type": "string",
                    "description": "The city and state, e.g. San Francisco, CA"
                  },
                  "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"]
                  }
                },
                "required": ["location"]
        }
    });

    assert_eq!(function.json, expected_result);
}
