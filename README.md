# Simplifying Ai Agents in Rust 🦀
There are hundred of Ai agent applications written Typescript and Python. Espionox is an attempt to make building Ai applications in Rust just as approachable.

## Current features
 - Easy API for initializing `Agent` structs and using them to prompt OpenAi's endpoints
 - Postgres database driven Long Term Memory behind `"long_term_memory"` feature flag
 - Embed and query files and directories using long term memory

## Getting started 
To get started add `espionox` to your `Cargo.toml`: 
```
espionox = { git = "https://github.com/voidKandy/espionox" }
```
espionox is not yet a cargo package, but soon adding it will be as easy as running `cargo add espionox` 

Afer adding espionox to your dependencies, add a `configuration` directory containing a `default.yaml` file to your root.
`default.yaml:`
```
language_model:
  model: "gpt-3.5-turbo-0613"
  api_key: ""


database: 
  host: 127.0.0.1 
  port: 6987
  username: "postgres"
  password: ""
  database_name: ""

```

***Only fill out the database section if you plan on using the long term memory feature***

## Using Long term memory 
Soon a docker image for espionox will be made for easy configuration of the postgres database, but as of right here are the steps to configure the database.
1. Create a Postgres server 
2. Initialize a database with the following migration:
```
BEGIN;

    CREATE EXTENSION IF NOT EXISTS vector;
    
    CREATE TABLE IF NOT EXISTS threads (
        name TEXT NOT NULL PRIMARY KEY UNIQUE
    );

    CREATE TABLE IF NOT EXISTS files (
        id TEXT PRIMARY KEY,
        thread_name TEXT,
        filepath TEXT NOT NULL,
        parent_dir_path TEXT NOT NULL,
        summary TEXT NOT NULL,
        summary_embedding vector(384)
    );

    CREATE TABLE IF NOT EXISTS file_chunks (
        id TEXT PRIMARY KEY,
        parent_file_id TEXT NOT NULL,
        parent_filepath TEXT NOT NULL,
        idx smallint NOT NULL,
        content TEXT NOT NULL,
        content_embedding vector(384)
    );

    CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        thread_name TEXT,
        role TEXT NOT NULL,
        content TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS io (
        id TEXT PRIMARY KEY,
        input TEXT,
        output TEXT
    );

COMMIT;
```
3. Fill out the relavent section in `configuration/default.yaml` and add the ***DATABASE_URL*** to your `.env` 

At the highest level of abstraction initializing an agent and prompting for a response can be done in a few lines of code: 
```
let mut agent = espionox::agent::Agent::default();
let prompt = "Hello chat agent!";
let response = agent.prompt(prompt).await.unwrap();
println!("{}", response);
```
Docs are not yet written, for more info on the API check out the tests folder or consult this [WIP example GUI](https://github.com/voidKandy/espionox_egui_demo/tree/master)



Please feel free to reach out with any questions, suggestions, issues or anything else :)
