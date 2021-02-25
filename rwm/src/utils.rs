use std::fs;

use anyhow::{Context, Result};
use x11rb::protocol::xproto::KeyButMask;

use crate::states::{TagState, WinState};

pub(crate) fn clean_mask(mask: u16) -> u16 {
    // TODO: num lock is not always Mod2, find a way to get that dynamically
    mask & !(KeyButMask::Mod2 | KeyButMask::Lock)
        & (KeyButMask::Shift
            | KeyButMask::Control
            | KeyButMask::Mod1
            | KeyButMask::Mod2
            | KeyButMask::Mod3
            | KeyButMask::Mod4
            | KeyButMask::Mod5)
}

pub(crate) fn clean_up() -> Result<()> {
    fs::remove_file("/tmp/rwm.sock").context("Failed to remove socket")?;

    Ok(())
}

/// Get all the visible tags only
pub(crate) fn visible(tags: &[TagState]) -> Vec<TagState> {
    tags.iter().filter(|tag| tag.visible).copied().collect()
}

/// Returns true if the window is in a tag that's visible
pub(crate) fn is_visible(win: &WinState, tags: &[TagState]) -> bool {
    tags.iter()
        .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
}
