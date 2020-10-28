use x11rb::protocol::xproto::KeyButMask;

pub fn clean_mask(mask: u16) -> u16 {
    // TODO: num lock is not always Mod2, find a way to get that dynamically
    mask & !(KeyButMask::Mod2 | KeyButMask::Lock) & (KeyButMask::Shift | KeyButMask::Control | KeyButMask::Mod1 | KeyButMask::Mod2 | KeyButMask::Mod3 | KeyButMask::Mod4 | KeyButMask::Mod5)
}

