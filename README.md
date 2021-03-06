# rwm - The Rusty Window Manager
rwm is a dynamic tilling (and floating) window manager for X11 using the tag system of dwm and the server/client style of bspwm.
It's written in Rust using [bindings](https://crates.io/crates/x11rb) to the xcb library.

rwm strives to be easy to use and configure through a simple config file.  
The config file is currently changing a lot but here is how it looks like at the moment of writing:
```rust
(
    border_width: 4,
    focused_border_color: "#0000FF",
    normal_border_color: "#D3D3D3",
    mod_key: "Mod 1",
    // The order of the layouts is the order in which they will cycle
    layouts: [
        MonadTall,
        Grid,
        Floating,
    ],
    // Focus follows the cursor
    follow_cursor: true, 
    gap: 4,
    // Rules can be based either on WM_CLASS or WM_NAME using ClassName() or WMName() respectively
    rules: [
        // Put Firefox in both tag 1 and tag 2
        ClassName("firefox", [(3), (2)]), 
        // Put Alactitty in tag 3 only
        ClassName("Alacritty", [(3)]), 
    ],
)
```
You might notice the unfamiliar syntax. It's called [RON](https://github.com/ron-rs/ron)

## To-do
This is **not a viable window manager yet**, but it's coming together nicely.  
See [To-do](./TODO.md) for things that need to happen.
