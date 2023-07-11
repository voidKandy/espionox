# Consoxide

Breathing life into your console sessions with Ai!

Consoxide puts the power of LLMs into your very own console! All you need is to run the `start.sh` file to get the tmux session running.

## Current features
* Agent 'watches' your commands in the agent tmux window. It can offer help with any errors that get run through that terminal window.
* Dynamically updates context based on your actions
* Summarize & embed files or entire directories

## In development
* Long term memory using `pgvector` in a postgres SQL server
* User tailored configuration
* The ability to add arbitrary directories anywhere on your harddrive to long term memory
  * Dynamicly update when changes are made
