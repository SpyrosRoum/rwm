#!/usr/bin/bash

/usr/bin/Xephyr :1 -ac -br -noreset -screen 1152x720 &
sleep 1s # Otherwise the display doesn't have time to start, or something like that and it crashes
DISPLAY=:1.0 /usr/bin/alacritty &
sleep 1s
DISPLAY=:1.0 ./target/debug/rwm
