#!/bin/bash

session="tmux-monitor"

tmux new-session -d -s $session

window=0
tmux rename-window -t $session:$window 'monitor'

# Run cargo run in the Tmux session
tmux send-keys -t $session:$window 'cargo run' Enter

tmux attach-session -t $session
