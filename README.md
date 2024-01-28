# Simplifying Ai Agents in Rust 🕵🏼

There are hundred of Ai agent applications written Typescript and Python. Espionox is an attempt to make building Ai applications in Rust just as approachable.

## Why would I use Espionox?

- Making an LLM application in Rust
- Experimenting with with complex 'prompt flows' such as Chain/Tree of thought

## Overview

The backbone of `espionox` is the `Agent` struct, which contains:

- Model
- Memory

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
use espionox::environment::Environment;

let env_name = "demo-env";
let api_key: &str = "your-openai-api-key";
let env = Environment::new(Some(env_name), Some(api_key));
```

Afer adding espionox to your dependencies, add a `espionox-config` directory containing a `env` directory to your root.
Add a `default.yaml:` file to `espionox-config/env`:

```
use espionox::environment::Agent;

let agent = Agent::default();
let agent_name = "my agent";
let agent_handle = env.insert_agent(Some(agent_name), agent).await.unwrap();
```

database:
  host: 127.0.0.1
  port: 6987
  username: "postgres"
  password: ""
  database_name: ""

```
env.spawn().await.unwrap()
```

**_You only need to fill out the database section if you're using the long term memory feature_**

```

Check out the [this example repo](https://github.com/voidKandy/espionox_egui_demo/tree/master) for how to pull from the espionox docker image. Essentially, all you need to do is create an `.env` file with all the info provided in the example's `.env`. Then, create a `docker-compose.yaml` with all the relevant info. Run `docker-compose build` and finally `docker-compose up` to get your database running.
```
let noti = env.notifications.wait_for_notification(&ticket).await.unwrap();
let message: &Message = noti.extract_body().try_into().unwrap();
```

Completions can also be returned as streams if the `request_stream_completion` method is used:

```
let stream_handler: &ThreadSafeStreamCompletionHandler = noti.extract_body().try_into().unwrap();
let mut handler = stream_handler.lock().await;

let mut whole_message = String::new();
while let Some(CompletionStreamStatus::Working(token)) = handler
    .receive(&handle.id, environment.clone_sender())
    .await
{
    whole_message.push_str(&token);
}
println!("GOT WHOLE MESSAGE: {}", whole_message);
```

## Listeners

One of Espionox's best offerings is the EnvListener trait

```
pub trait EnvListener: std::fmt::Debug + Send + Sync + 'static {
    /// Returns Some when the listener should be triggered
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage>;
    /// method to be called when listener is activated
    fn method<'l>(
        &'l mut self,
        trigger_message: &'l EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> Pin<Box<dyn Future<Output = Result<(), DispatchError>> + Send + Sync + 'l>>;
    /// Optional method to replace the triggering message with another
    fn mutate<'l>(&'l mut self, origin: EnvMessage) -> EnvMessage {
        origin
    }
}
```

Any struct implementing this trait can be added to an Environment using the `add_listener` method. EnvListeners can be used to enforce self consistency in creative ways. Check the examples directory for examples of listeners in action.

espionox is very early in development and everything in the API may be subject to change
Please feel free to reach out with any questions, suggestions, issues or anything else :)
