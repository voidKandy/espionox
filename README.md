# Simplifying Ai Agents in Rust 🦀
There are hundred of Ai agent applications written Typescript and Python. Espionox is an attempt to make building Ai applications in Rust just as approachable.

## Current features
 - Easy API for initializing `Agent` structs and using them to prompt OpenAi's endpoints
 - Postgres database driven Long Term Memory behind `"long_term_memory"` feature flag
 - Embed and query files and directories using long term memory

## Getting started 
To get started simply add `espionox` to your `Cargo.toml`: 
```
espionox = { git = "https://github.com/voidKandy/espionox" }
```
espionox is not yet a cargo package, but soon adding it should be as easy as running `cargo add espionox` 

At the highest level of abstraction initializing an agent and prompting for a response can be done in a few lines of code: 
```
let mut agent = espionox::agent::Agent::default();
let prompt = "Hello chat agent!";
let response = agent.prompt(prompt).await.unwrap();
println!("{}", response);
```
Docs are not yet written, for more info on the API check out the tests folder or consult this [WIP example GUI](https://github.com/voidKandy/espionox_egui_demo/tree/master)

Please feel free to reach out with any questions, suggestions, issues or anything else :)
