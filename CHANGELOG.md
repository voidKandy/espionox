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
