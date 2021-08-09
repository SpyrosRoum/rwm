#!/usr/bin/sh
set -eu

fake_monitors=0
release=0

for arg in "$@"
do
  case "$arg" in
    --fake-monitors)
      fake_monitors=1
      ;;
    --release)
      release=1
      ;;
    *)
      ;;
  esac
done

if [ $release -ne 0 ]; then
  if [ $fake_monitors -ne 0 ]; then
    (set -x; cargo build --release --features fake_monitors)
  else
    (set -x; cargo build --release)
  fi
elif [ $fake_monitors -ne 0 ]; then
  (set -x; cargo build --features fake_monitors)
else
  (set -x; cargo build)
fi

# NOTE: Resizing doesn't work correctly in Xephyr,
# to be exact the pointer won't warp to the bottom right corner of the window.

/usr/bin/Xephyr :1 -ac -br -noreset -screen 1920x1080 2> /dev/null &
sleep 1s # Otherwise the display doesn't have time to start, or something like that and it crashes
DISPLAY=:1.0 /usr/bin/alacritty &
sleep .5s # A small delay for alacritty to start otherwise rwm won't see it. Maybe I should put a delay in rwm itself?

if [ $release -ne 0 ]; then
  (set -x; DISPLAY=:1.0 ./target/release/rwm)
else
  (set -x; DISPLAY=:1.0 ./target/debug/rwm)
fi
