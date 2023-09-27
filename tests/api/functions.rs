use super::helpers::init_test;
use consoxide::language_models::openai::functions::{
    CustomFunction, Function, Property, PropertyInfo,
};
use serde_json::json;

pub fn weather_test_function() -> CustomFunction {
    let location_info = PropertyInfo::new(
        "description",
        json!("The city and state, e.g. San Francisco, CA"),
    );
    let unit_info = PropertyInfo::new("enum", json!(["celcius", "fahrenheit"]));

    let location_prop = Property::build_from("location")
        .return_type("string")
        .add_info(location_info)
        .finished();
    let unit_prop = Property::build_from("unit")
        .return_type("string")
        .add_info(unit_info)
        .finished();

    CustomFunction::build_from("get_current_weather")
        .description("Get the current weather in a given location")
        .add_property(location_prop, true)
        .add_property(unit_prop, false)
        .finished()
}

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
                    "enum": ["celcius", "fahrenheit"]
                  }
                },
                "required": ["location"]
        }
    });

    assert_eq!(function.json, expected_result);
}
