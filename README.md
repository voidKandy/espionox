# 🦀 Consoxide 🦀


<img width="1508" alt="Screenshot 2023-08-05 at 3 35 27 PM" src="https://github.com/voidKandy/Consoxide/assets/121535853/cd59f138-4c3a-4276-93ba-ee8d381ab539">

# Simplifying Ai Agents in Rust
Everyone is creating Ai agent applications using typescript and python. So this repo is an attempt to make doing the same in Rust just as approachable.




## Current features
* Openai completions api easily accesible through an io device
* Agent which can switch Memory modes:
   * Temporary: for simple queries like file summaries or anything doesn't require a large context window
   * Short Term: the cached conversation for the current runtime
   * Long Term: Database connected Conversation
 * Memorable trait to make objects 'memorizable' by putting them in the context of a message

## In development
* Streaming completions
* SOPs
