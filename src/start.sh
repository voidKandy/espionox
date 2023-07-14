#!/bin/bash
source .env

session=$TMUX_SESSION_NAME
tmux new-session -d -s $session

tmux split-window -h
tmux run-shell -b -d 0 -t $OUTPUT_PANE 'echo "                                              --Consoxide--"'

tmux new-window 
tmux new-window 

tmux rename-window -t $session:0 'agent'
tmux rename-window -t $session:1 'dev'
tmux rename-window -t $session:2 'watched'

tmux select-window -t $session:1
tmux send-keys -t . nv Enter

# tmux kill-window -t  $session:2

tmux attach-session -t $session
