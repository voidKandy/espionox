
# 🕵️

# Simplifying Ai Agents in Rust 🦀
There are hundred of Ai agent applications written Typescript and Python. So this repo is an attempt to make doing the same in Rust just as approachable.


<img width="1122" alt="Screenshot 2023-09-06 at 6 41 22 PM" src="https://github.com/voidKandy/Consoxide/assets/121535853/16006fc5-85e2-4bc6-bdf5-aa356e90234f">

## Agent
The **Agent** struct glues the **Context** and the **Gpt** structs together, allowing **Gpt** to query it's associated endpoint using the **Context**.
## Context
**Context** handles the relationship between the selected memory variant and the current message buffer. There are 3 memory variants: 
* Temporary - The message buffer exists only as long as it is in-scope
* ShortTerm - The message buffer persists for as long as the current runtime
* LongTerm - The message buffer persists until the database is cleared
### Database connection
Consoxide uses Postgres under the hood and is fully capable of storing and querying vector embeddings. While eventually I would like to add support for embedding a lot more, currently only file embeddings are supported. Files are chunkified and summarized, embedding both full file summaries and the content of each file chunk.
### Interfaces
As of right now only a rudimentary terminal interface is written and working, but an egui powered GUI is in the works. 

### Getting started
You will need to host a Postgres database on your own machine. After cloning this repo, fill out the example `configuration/default_example.yaml` with your OpenAi Api key and all relavent database information. Make sure to rename it to `default.yaml`. Then run `scripts/init_db.sh` to get started. You are now ready to use Consoxide! Try to run `cargo run --bin terminal` to check out the terminal interface.

Please feel free to reach out with any questions, suggestions, issues or anything else :)
