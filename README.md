# Simplifying Ai Agents in Rust 🕵🏼

`espionox` is an attempt to make building Ai applications in Rust just as approachable as it is with other libraries such as LangChain.

## Why would I use Espionox?

- Making an LLM application in Rust
- Experimenting with with complex 'prompt flows' such as Chain/Tree of thought

## Getting started

First you need to initialize an `Agent`
`Agent::new` accepts two arguments: 
1. Optional content of a system prompt, if this is left `None` your agent will have no system prompt
2. An api key for whichever provider you use (As of writing, only OpenAi and Anthropic providers are supported).

```
let api_key = std::env::var("OPENAI_KEY").unwrap();
let agent = Agent::new(Some("This is the system message"), LLM::default_openai(api_key));
```

Now, In order to prompt your agent you will call `do_action` on it (method name WIP)

```
let response: String = agent
    .do_action(io_completion, (), Option::<ListenerTrigger>::None)
    .await
    .unwrap();
```
This may look scary at first, lets look at `do_action`'s signature: 
```
pub async fn do_action<'a, F, Args, Fut, R>(
    &'a mut self,
    f: F,
    args: Args,
    trigger: Option<impl Into<ListenerTrigger>>,
) -> AgentResult<R>
where
    F: for<'l> FnOnce(&'a mut Agent, Args) -> Fut,
    Fut: Future<Output = AgentResult<R>>
```
`do_action` takes 4 arguments:
1. the `Agent` which calls the method
2. an async function which mutates the agent and returns some result
3. optionally arguments for the aformentioned function 
4. An optional trigger for a listener (We'll get to this)

So, in our call to `do_action` earlier, we passed the function `io_completion`, an empty argument and None.
`espionox` provides the following helper functions for getting completions or embeddings:
* `get_embedding`
* `io_completion`
* `stream_completion`
* `function_completion`
We used one of these functions, but we could have just as easily defined our own `io_completion` function and passed it when we called `do_action`

## Listeners

One of Espionox's best offerings is the `AgentListener` trait:

```
pub trait AgentListener: std::fmt::Debug + Send + Sync + 'static {
    fn trigger<'l>(&self) -> ListenerTrigger;
    /// needs to be wrapped in `Box::pin(async move {})`
    fn async_method<'l>(&'l mut self, _a: &'l mut Agent) -> ListenerCallReturn<'l> {
        Box::pin(async move { Err(ListenerError::NoMethod.into()) })
    }
    fn sync_method<'l>(&'l mut self, _a: &'l mut Agent) -> AgentResult<()> {
        Err(ListenerError::NoMethod.into())
    }
}
```
You will notice 3 methods:
1. `trigger`: this is how you define when the listener will be triggered. Think of it like an ID. `ListenerTrigger` has 2 variants: 
    * `ListenerTrigger::String(String)`
    * `ListenerTrigger::Int(i64)`
    Remember the `trigger` argument to `do_action`? Ensure a listener is triggered when `do_action` is called by passing a matching `ListenerTrigger`.
2. `async_method`
3. `sync_method`
Each `async_method` and `sync_method` are where you define WHAT the listener will actually do when it's triggered. THESE ARE MUTUALLY EXCLUSIVE, only ONE of these methods should be implemented. 
Any struct implementing this trait can be inserted into an agent using `Agent::insert_listener`. 

### How do you even use a listener??

The utility of listeners may not be immediately obvious to you, but it can be used to create self consistency mechanisms, prompt chains or even RAG pipelines.
Check the examples directory for more information on `AgentListener`

espionox is very early in development and everything in the API may be subject to change Please feel free to reach out with any questions, suggestions, issues or anything else :)
