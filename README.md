# Consoxide 


# Simplifying Ai Agents in Rust 🦀
There are hundred of Ai agent applications written Typescript and Python. So this repo is an attempt to make doing the same in Rust just as approachable.


<img width="1122" alt="Screenshot 2023-09-06 at 6 41 22 PM" src="https://github.com/voidKandy/Consoxide/assets/121535853/16006fc5-85e2-4bc6-bdf5-aa356e90234f">
#### Agent
**Agents** are easy to initialize structs which handle all database interractions as well as cached messages. Fully capable of adding/removing/editing files stored in a Postgres database. All files are chunkified and embedded for easy vector database querying.


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
