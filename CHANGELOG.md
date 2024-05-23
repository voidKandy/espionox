# v0.1.1

## Overview of changes

Changed `Message::new` method to four different methods:

- `new_system`
- `new_user`
- `new_assistant`
- `new_other`
- Updated Examples, basic RAG implementation now included
- `ToMessage` trait no longer requires structs implementing it implement `Display` trait, also removed `role` method as it's adds unnecesary redundancy
- new method `request_state` added to `AgentHandle` for getting the current cache state of the associated agent
- Added experimental `Surfer` and `Vision` agents
- Listener trait changed slightly; `mutate` method no longer exists, any mutations to the trigger message should be done in `method`, which now returns an `EnvMessage`

# v0.1.11

## HOTFIX

Merge removed `is_running` method from `Environment` this fixes that

# v0.1.2

This is a relatively big update. Adding a lot of quality of life changes. As well as a few feature.

## Quality Of Life & Misc Changes

- Implemented `ToMessage` for `String`
- `request_state` was accidentally removed in merge of v0.1.1, Added it back.
- `clone_sans_system_prompt` now returns an option, returns Some if the vector isn't empty
- `request_cache_push` method was added to `AgentHandle`. It sends a `PushToCache` request to the env. Returns `()` as `EnvRequest::PushToCache` does not have a ticket.
- updated `GPT3_MODEL_STR` to reflect the most recent GPT 3 model available
- Improved errors; Errors streamlined, debugging should be easier
- Removed filesys feature; This feature was pretty pointless
- Added tools feature and put dependencies behind feature flag
- Moved `environment::agent` module to its own module: `crate::agents`
- Removed `configuration` module

## New features

- Added `IndependentAgent` struct. Should make it a little easier to use agents outside of an env. Supports both io and function completions. Does not support streamed completions
- Added Surfer and vision features in tools directory. Still very experimental, use at your own risk

# v0.1.21

## HOTFIX

- IncorrectTrigger error added to listenererror
- NoAgent error added to listenererror

# v0.1.22

## Breaking changes

`MessageVector` renamed to `MessageStack` for more accurate naming
Also added `MessageStackRef`, which contains references to messages within a stack owned elsewhere
This struct also has `len`, `pop`, and `filter_by` methods

```
#[derive(Clone, Debug, PartialEq)]
pub struct MessageStackRef<'stack>(Vec<&'stack Message>);

impl<'stack> From<Vec<&'stack Message>> for MessageStackRef<'stack> {
    fn from(value: Vec<&'stack Message>) -> Self {
        Self(value)
    }
}

impl Into<MessageStack> for MessageStackRef<'_> {
    fn into(self) -> MessageStack {
        MessageStack(self.0.into_iter().map(|m| m.clone()).collect())
    }
}
```

Added/Removed methods from what is now `MessageStack`
_Added_:

- `pop` - which has an optional role parameter for popping last message matching
- `mut_filter_by` mutates `MessageStack` in place
- `ref_filter_by` returns `MessageStackRef` of filtered messages
- `filter_by` method was also added to `MessageStackRef`

_Removed_ (`filter_by` deprecated all of the following):

- `clone_sans_system_prompt`
- `clone_system_prompt`
- `chat_count`
- `reset_to_system_prompt`

## Other changes

Reorganized OpenAi model stuff to prepare for support of more models.
Added embedding models for openai.
Put all huggingface stuff behind 'bert' feature flag

# v0.1.23

## Removed Unused Dependencies

Removed:
"async-recursion"
"byteorder"
"rand"
"serde-aux"
"serde_yaml"

## New Model traits BREAKING CHANGE

Anthropic models now supported
Model enpoint handlers have been completely redone to make it easier to expand model support in the future.
All supported models now implement the `EndpointCompletionHandler` trait. Because this trait is implemented
over generic `H` starting @ the new `LLMCompletionHandler` struct, the `H` generic has bubbled up all the way to `Environment`.

#### Model Provider

To support referring to specific providers this enum has been added:

```
pub enum ModelProvider {
    OpenAi,
    Anthropic,
}
```

#### Agent

`Default` no longer implemented
`model` field changed to `completion_handler` field.
The process of initializing an agent now requires the initialization of an `LLMCompletionHandler`. For Example:

```
let handler = LLMCompletionHandler::<OpenAiCompletionHandler>::default_openai();
let agent = Agent::new(
    "test",
    handler
);
```

#### Environment

Because of the added support for more models, api keys are now stored in a hashmap:
`HashMap<ModelProvider, String>`
`Environment::new()` method has changed to take a hashmap instead of an `Option<String>`

```
let open_key = std::env::var("OPENAI_KEY").unwrap();
let anth_key = std::env::var("ANTHROPIC_KEY").unwrap();

let mut keys = HashMap::new();
keys.insert(ModelProvider::OpenAi, api_key);
keys.insert(ModelProvider::Anthropic, api_key);

Environment::new(Some("testing"), keys)
```

# v0.1.24

## Model Endpoint handler changed

`H` removed LLMCompletionHandler is now an enum, no need for type generics
`LLM` struct can get either completions or embeddings, and eventually will support vision models
The interface has been built to easily add new models
EndpointCompletionHandler now InferenceEndpointHandler, Which stores
CompletionEndpointHandler and EmbeddingEndpointHandler
Agents which get embeddings is now possible, though Dispatch channel hasn't been updated to use
embedding agents, As i currently see little user for that. So for now, only IndependantAgents can be used for Embedding agents

## Agents

Agent::new now takes an optional system prompt str instead of forcing you to provide one
`completion_handler` has been made public only to the crate

## Independent Agents

`agent` field has been made private. New `mutate_agent_cache` function allows you
to pass a closure to mutate the cache of the inner agent.
Supported endpoint response getters are their own methods

## Messages/MessageStack

Removed general `Into<Vec<Value>>` for MessageStack to allow each provider to have it's own implementation
Added `IntoIter` to MessageStack
Added `From<Vec<Value>>` to MessageStack

### Change to `request_cache_push`

Now takes a `Message` instead of a `ToMessage` and a Role

## Notification stack

Now has async `pop_front` and `pop_back` methods. These simply aquire write locks and then pop either front or back from the inner
`VecDeque`.
`NotificationStack` is also no longer a wrapper for `Arc<RwLock`, it is simply a wrapper for `VecDeque`. There is a named type `RefCountedNotificationStack` which is an alias for
`Arc<RwLock`NotificationStack>>`

## EnvHandle

New `EnvHandle` struct added to modularize making requests to a running environment
`Environment::spawn_handle` returns `EnvHandle`. When this method is run, the `EnvHandle` takes ownership of `Dispatch` and `Listeners` Vector.
Rather than `finalize_dispatch`, `EnvHandle` has a method called `finish_current_job`, which joins the thread handle and returns an owned NotificationStack.

## AgentStateUpdate

Agent state update now returns a ticket number from the `ticket_number` method.

## Anthropic Handler

Because anthropic needs user/assistant pairs, `agent_cache_to_json` has been added for each completion handler to have it's own way of getting a vec of values from `MessageStack`

# v0.1.25

ModelParameters fields made public

Removed &mut requirement for AgentHandle methods
added Serialize/Deserialize to ModelProvider

`MessageRole::Other` changed to allow user's to pick what role it's coerced to when rendered
A few things had to happen to allow for this.
added `actual()` method to return actual message role to account for the fact that `Other` is essentially an alias
`From` implementations are now `TryFrom`
a new enum `OtherRoleTo` to allow users to pick which the messages get coerced to

# v.0.1.3

Added find by to noti stack
removed environment system!

`LLM` struct now contains `reqwest::Client` & api key

Listeners are now put on agents directly
