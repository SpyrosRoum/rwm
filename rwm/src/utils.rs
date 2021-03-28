use std::fs;

use {
    anyhow::{anyhow, Context, Result},
    x11rb::{
        protocol::xproto::{AtomEnum, ConnectionExt, KeyButMask, Window},
        rust_connection::RustConnection,
    },
};

use crate::states::{TagState, WinState};

pub(crate) fn clean_mask(mask: u16) -> u16 {
    // ToDo: I think num lock is not always Mod2, find a way to get that dynamically
    let num_lock = u16::from(KeyButMask::MOD2 | KeyButMask::LOCK);
    (mask & !num_lock)
        & u16::from(
            KeyButMask::SHIFT
                | KeyButMask::CONTROL
                | KeyButMask::MOD1
                | KeyButMask::MOD2
                | KeyButMask::MOD3
                | KeyButMask::MOD4
                | KeyButMask::MOD5,
        )
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

/// A help function to get the WM_TRANSIENT_FOR hint for the given window
pub(crate) fn get_transient_for(conn: &RustConnection, win_id: Window) -> Result<Option<Window>> {
    Ok(conn
        .get_property(
            false,
            win_id,
            AtomEnum::WM_TRANSIENT_FOR,
            AtomEnum::WINDOW,
            0,
            1,
        )?
        .reply()?
        .value32()
        .ok_or_else(|| anyhow!("Wrong format"))?
        .next())
}
