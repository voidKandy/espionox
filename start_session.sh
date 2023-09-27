#!/bin/bash

session="espionox"
cd ~/Documents/GitHub/espionox/
# Check if the session exists
tmux has-session -t $session 2>/dev/null

if [ $? != 0 ]; then
    tmux new-session -d -s $session

    tmux new-window 
    tmux rename-window -t $session:0 'zsh'
    tmux rename-window -t $session:1 'neovim'

    tmux send-keys -t $session:0.0 'export RUST_LOG="sqlx=error,info"' Enter
    tmux send-keys -t $session:0.0 'export TEST_LOG=enabled' Enter

    tmux send-keys -t $session:1 'nv' Enter
fi

tmux select-window -t $session:1
tmux attach-session -t $session

