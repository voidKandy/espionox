# Simplifying Ai Agents in Rust 🕵🏼

`espionox` is an attempt to make building Ai applications in Rust just as approachable as it is with other libraries such as LangChain.

## Why would I use Espionox?

- Making an LLM application in Rust
- Experimenting with with complex 'prompt flows' such as Chain/Tree of thought

## Usage

First you need to initialize an `Agent`

`Agent::new` accepts two arguments: 
1. Optional content of a system prompt, if this is left `None` your agent will have no system prompt
2. A `CompletionModel` whichever provider you wish to use (As of writing, only OpenAi and Anthropic providers are supported).

```rust
use espionox::prelude::*;

let api_key = std::env::var("OPENAI_KEY").unwrap();
let agent = Agent::new(Some("This is the system message"), CompletionModel::default_openai(api_key));
```
Now, In order to prompt your agent you will call any of the following 3 methods: 
+ `io_completion`
+ `stream_completion`
+ `function_completion`

### Io Completion
```rust
impl Agent {
    pub async fn io_completion(&mut self) -> AgentResult<String>;
}
```
This is the most straightforward way to get a completion from a model, it will simply request a completion from the associated endpoint with the models' current context.

### Stream Completion
```rust
impl Agent {
    pub async fn stream_completion(&mut self) -> AgentResult<ProviderStreamHandler>;
}
```
This will return a stream response handler object that needs to be polled for tokens, for example:
```rust
let mut response: ProviderStreamHandler = a.stream_completion().await.unwrap();
while let Ok(Some(res)) = response.receive(&mut a).await {
    println!("Received token: {res:?}")
}
```
When the stream completes, the finished message will *automatically* be added to the agent's context, so you *do not* have to worry about making sure the agent is given the completed response

### Function Completion
> Currently only available with `OpenAi` models
```rust
impl Agent {
    pub async fn function_completion(&mut self, function: Function) -> AgentResult<serde_json::Value>;
}
```

This is a feature built on top of OpenAi's [function calling API](https://cookbook.openai.com/examples/how_to_call_functions_with_chat_models). Instead of needing to write functions as raw JSON, `espionox` allows you to use it's own language which get's compiled into the correct `JSON` format when fed to the model.
The structure of a function is as follows: 
```
<function name>([<argname: type>])
    i = <description of what function does>
    [<optional descriptions of each arg>]
```
If an arg's name is followed by a `!`, it means that argument is required.

Supported argument types: 
+ `bool`
+ `int`
+ `string`
+ `enum` - with variants defined in single quotes, separated by `|`

##### Weather Function Example
Imagine you want to use the function given in [OpenAi's example](https://cookbook.openai.com/examples/how_to_call_functions_with_chat_models)
 ```json
{
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
}
```
Instead of hand writing the above `JSON` object, you can use espionox's function language
```rust
let weather_function_str = r#"
    get_current_weather(location!: string, unit: 'celcius' | 'farenheight') 
            i = 'Get the current weather in a given location'
            location = 'the city and state, e.g. San Francisco, CA'
"#;
let weather_function = Function::try_from(weather_function_str);
let json_response = agent.function_completion(weather_function).await?;
```
The returned `serde_json::Value` contains the specified args and their values as key value pairs. For example, `json_response` might look something like:
```json
{
    "location": "San Francisco, CA",
    "unit": "fahrenheit"
}
```
___
`espionox` is very early in development and everything  may be subject to change Please feel free to reach out with any questions, suggestions, issues or anything else :)
#### [Most Recent Change](/CHANGELOG.md#v0.1.40)

