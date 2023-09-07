# Consoxide 


<img width="1508" alt="Screenshot 2023-08-05 at 3 35 27 PM" src="https://github.com/voidKandy/Consoxide/assets/121535853/cd59f138-4c3a-4276-93ba-ee8d381ab539">

# Simplifying Ai Agents in Rust 🦀
There are hundred of Ai agent applications written Typescript and Python. So this repo is an attempt to make doing the same in Rust just as approachable.

<img width="1122" alt="Screenshot 2023-09-06 at 6 41 22 PM" src="https://github.com/voidKandy/Consoxide/assets/121535853/16006fc5-85e2-4bc6-bdf5-aa356e90234f">

## Straightforward Agent interface

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
