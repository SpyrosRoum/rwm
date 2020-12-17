# rwm - The Rusty Window Manager
rwm is a dynamic tilling (and floating) window manager for X11 using the tag system of dwm and the server/client style of bspwm.
It's written in Rust using [bindings](https://crates.io/crates/x11rb) to the xcb library.


# DO NOT USE THIS YET
This is not a viable window manager yet, but it's coming together nicely.
It still has many missing features but the greatest issues are:
* No multi monitor support at all
* zero coverage of ICCCM or EWMH

There are other issues too but since the project is under active development and improvements/fixes/features are happening constantly, I don't think they are worth mentioning.
