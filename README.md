# Simplifying Ai Agents in Rust 🦀
There are hundred of Ai agent applications written Typescript and Python. Espionox is an attempt to make building Ai applications in Rust just as approachable.

## Current features
 - Easy API for initializing `Agent` structs and using them to prompt OpenAi's endpoints
 - Runtime Short Time Memory
 - Postgres database driven Long Term Memory behind `"long_term_memory"` feature flag
 - Embed and query files and directories using long term memory

## Getting started 
To get started add `espionox` to your `Cargo.toml`: 
```
espionox = { git = "https://github.com/voidKandy/espionox", branch = "stable" }
```

Afer adding espionox to your dependencies, add a `espionox-config` directory containing a `env` directory to your root.
Add a `default.yaml:` file to `espionox-config/env`: 
```
language_model:
  default_model: "gpt-3.5-turbo-0613"
  api_key: ""


database: 
  host: 127.0.0.1 
  port: 6987
  username: "postgres"
  password: ""
  database_name: ""

```

***Only fill out the database section if you're using the long term memory feature***

## Using Long term memory 
Check out the [this example repo](https://github.com/voidKandy/espionox_egui_demo/tree/master) for how to pull from the espionox docker image. Essentially, all you need to do is create an `.env` file with all the info provided in the example's `.env`. Then, create a `docker-compose.yaml` with all the relevant info. Run `docker-compose build` and finally `docker-compose up` to get your database running.

## API overview
At the highest level of abstraction initializing an agent and prompting for a response can be done in a few lines of code: 
```
let mut agent = espionox::agent::Agent::default();
let prompt = "Hello chat agent!";
let response = agent.prompt(prompt).await.unwrap();
println!("{}", response);
```
Docs are not yet written, for more info on the API check out the tests folder or consult this [WIP example GUI](https://github.com/voidKandy/espionox_egui_demo/tree/master)

espionox is very early in development and everything in the API may be subject to change
Please feel free to reach out with any questions, suggestions, issues or anything else :)
