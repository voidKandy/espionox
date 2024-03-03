# v0.1.1

## Overview of changes

- Changed `Message::new` method to four different methods:
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
