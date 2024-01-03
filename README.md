# Simplifying Ai Agents in Rust 🕵🏼
There are hundred of Ai agent applications written Typescript and Python. Espionox is an attempt to make building Ai applications in Rust just as approachable.

## Overview
The backbone of `espionox` is the `Agent` struct, which contains:
   * Model 
   * Memory 
   * Observer 
### Model
Currently only Gpt3.5 and 4 are supported, open source model integration is a planned future feature.
### Memory
Contains short term memory cache, the backbone of the agent's context. `Memory` also gives developers control over the context size upper bounds, and how the cache should react when it gets oversized. For example, an agent can be told to summarize the context window every 50 messages to ensure the context size never exceeds 50 messages.
If the `long_term_memory` feature is enabled, `Memory`'s `long_term` field allows the agent to be connected to a database.
### Observer
This is `Agent`'s only optional field, if used, `Observer` is another agent which watches the base agent and makes changes to incoming and/or outgoing messages. A planned future feature is to allow observers to be used to give developers an oportunity to inject callbacks into the conversation pipeline as well as mutate the base agent as needed.

## Getting started 
To get started add `espionox` to your `Cargo.toml`: 
```
espionox = { git = "https://github.com/voidKandy/espx-lib", branch = "stable" }
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

***You only need to fill out the database section if you're using the long term memory feature***

## Using Long term memory 
Check out the [this example repo](https://github.com/voidKandy/espionox_egui_demo/tree/master) for how to pull from the espionox docker image. Essentially, all you need to do is create an `.env` file with all the info provided in the example's `.env`. Then, create a `docker-compose.yaml` with all the relevant info. Run `docker-compose build` and finally `docker-compose up` to get your database running.

Docs are not yet written, for more info on the API check out the tests folder or consult this [WIP example GUI](https://github.com/voidKandy/espionox_egui_demo/tree/master)

espionox is very early in development and everything in the API may be subject to change
Please feel free to reach out with any questions, suggestions, issues or anything else :)
