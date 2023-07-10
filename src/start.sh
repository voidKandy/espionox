#!/bin/bash

session="tmux-monitor"

tmux new-session -d -s $session

tmux split-window -h
tmux new-window 

tmux rename-window -t $session:0 'agent'
tmux rename-window -t $session:1 'dev'

tmux kill-window -t  $session:2

tmux attach-session -t $session
