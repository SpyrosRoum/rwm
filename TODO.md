This is a list of things that I want to do/fix/improve. There may be more things that I want to do and there probably are more bugs. 

# To-do
- [X] Introduce logs
- [ ] Add scratch pads
- [ ] Add support for a bar
- [X] Multi-monitor support (RandR)
  - [X] Add commands for moving windows to other monitors and giving focus to other monitors
  - [ ] Detect when monitors get (un)plugged and handle it accordingly
- [ ] Better ICCCM/EWMH coverage
- [ ] Fake fullscreen (Allow windows to fullscreen into the space currently given to them)
- [ ] Actual fullscreen (A command toggle to make a window fullscreen)
- [ ] Probably more layouts, here are some ideas 
  (maybe not all of them will happen but layouts are easy to implement and PRs are welcome of course):
  - [ ] [Deck](https://dwm.suckless.org/patches/deck)
  - [ ] Monad Wide (Like monad tall but slaves go under main)
  - [ ] Bsp (See bspwm)
  - [ ] Monocle (See dwm's default monocle layout)

# Fixes
- [X] All windows spawn in the first monitor
- [ ] When you press mod + right click to resize a window for some reason the window becomes a little bigger in width and height
- [ ] In various places I need to know the mod mask for numlock and from what I understand this is not always Mod2,
so find a way to get that
- [ ] xkill doesn't actually kill windows.
