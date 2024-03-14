# Simplifying Ai Agents in Rust 🕵🏼

Espionox is an attempt to make building Ai applications in Rust just as approachable as it is with other libraries such as LangChain.

## Why would I use Espionox?

- Making an LLM application in Rust
- Experimenting with with complex 'prompt flows' such as Chain/Tree of thought

## Getting started

First, you'll want to create an environment.

```
let open_key = std::env::var("OPENAI_KEY").unwrap();
let anth_key = std::env::var("ANTHROPIC_KEY").unwrap();

let mut keys = HashMap::new();
keys.insert(ModelProvider::OpenAi, api_key);
keys.insert(ModelProvider::Anthropic, api_key);

let env_name = "MyEnv";

Environment::new(Some(env_name), keys)
```

Once an `Environment` has been instantiated, you can add agents to it

```
let agent_name = "my agent";
let init_prompt = "You are a helpful assistant";
let handler = LLMCompletionHandler::<OpenAiCompletionHandler>::default_openai();
let agent = Agent::new(
    init_prompt,
    handler
);
let agent_handle = env.insert_agent(Some(agent_name), agent).await.unwrap();
```

When `insert_agent` returns Ok, it will return an `AgentHandler`
After inserting any Agents or EnvListeners run this command to start running the environment:

```
env.spawn().await.unwrap()
```

Once the environment is running, the `AgentHandler` can be used to make completion requests

```
let message = Message::new_user("Hello!");
let ticket = agent_handle.request_io_completion(message).await.unwrap();
```

Get the response to your completion requests with the returned `ticket` UUid

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
    /// method to be called when listener is activated, must return an env message to replace input
    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn;
}
```
It looks simple, but this trait will allow you to create RAG pipelines, add tool use, and create self reflection techniques.
Think of the `EnvMessages` as events that can trigger specific things to happen to your agents.


espionox is very early in development and everything in the API may be subject to change
Please feel free to reach out with any questions, suggestions, issues or anything else :)
