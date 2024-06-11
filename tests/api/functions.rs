use serde_json::{json, Value};
// https://cookbook.openai.com/examples/how_to_call_functions_with_chat_models

#[test]
fn compile_functions_into_actual() {
    let str_to_compile = r#"get_current_weather(location!: string, unit: 'celcius' | 'farenheight') 
        i = 'Get the current weather in a given location'
        location = 'the city and state, e.g. San Francisco, CA'
        "#;

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

    let str_to_compile = r#"get_n_day_weather_forecast(location!: string, format!: 'celcius' | 'farenheight', num_days!: integer)
        location = 'the city and state, e.g. San Francisco, CA'
        format = 'The temperature unit to use. Infer this from the users location.'
        num_days = 'The number of days to forcast'
        "#;
    let expected_result = json!({
      "name": "get_n_day_weather_forecast",
              "description": "Get an N-day weather forecast",
              "parameters": {
                  "type": "object",
                  "properties": {
                      "location": {
                          "type": "string",
                          "description": "The city and state, e.g. San Francisco, CA",
                      },
                      "format": {
                          "type": "string",
                          "enum": ["celsius", "fahrenheit"],
                          "description": "The temperature unit to use. Infer this from the users location.",
                      },
                      "num_days": {
                          "type": "integer",
                          "description": "The number of days to forecast",
                      }
                  },
                  "required": ["location", "format", "num_days"]
              },
    });
}
