#!/usr/bin/bash

cargo build

# NOTE: Resizing doesn't work correctly in Xephyr,
# to be exact the pointer won't warp to the bottom right corner of the window.

/usr/bin/Xephyr :1 -ac -br -noreset -screen 1920x1080 &
sleep 1s # Otherwise the display doesn't have time to start, or something like that and it crashes
DISPLAY=:1.0 /usr/bin/alacritty &
sleep .5s # A small delay for alacritty to start otherwise rwm won't see it. Maybe I should put a delay in rwm itself?
DISPLAY=:1.0 ./target/debug/rwm
