#!/bin/bash
source .env

session=$TMUX_SESSION_NAME
tmux new-session -d -s $session

tmux split-window -h
tmux resize-pane -t $OUTPUT_PANE -R 15


tmux new-window 
tmux new-window 

tmux rename-window -t $session:0 'agent'
tmux rename-window -t $session:1 'dev'
tmux rename-window -t $session:2 'watched'

tmux send-keys -t $session:1 nv Enter

# tmux select-window -t $session:0
# echo OUTPUT_PANE_WIDTH=$(tmux display-message -p -t $OUTPUT_PANE "#{pane_width}") >> .env
# tmux run-shell -b -d 20 -t $OUTPUT_PANE 'echo $WIDTH_OF_OUTPUT_PANE'

tmux select-window -t $session:1
tmux attach-session -t $session

