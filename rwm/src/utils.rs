use std::fs;

use {
    anyhow::{anyhow, Context, Result},
    x11rb::{errors::ReplyOrIdError, protocol::xproto::*, rust_connection::RustConnection},
};

use crate::{
    mod_mask::XModMask,
    states::{TagState, WinState},
};

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
    // Socket is being deleted on Drop implementation for WmState so there is nothing to do here
    // Stays for potentially future use
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

pub(crate) fn grab_buttons(
    conn: &RustConnection,
    window: Window,
    mod_key: XModMask,
    focus: bool,
) -> Result<(), ReplyOrIdError> {
    conn.ungrab_button(ButtonIndex::ANY, window, ModMask::ANY)?;

    // This ugly line is needed because grab_button expects something that implements Into<u16>
    // but EventMask is u32
    let event_mask =
        u32::from(EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION);
    let mod_key = u16::from(mod_key);
    if focus {
        // We need to grab for our modifier key, for our mod key + numlock, mod + lock, and mod + numlock + lock
        for mask in std::array::IntoIter::new([
            0_u16,
            ModMask::LOCK.into(),
            ModMask::M2.into(), // ToDo `M2` might not always be numlock (see utils.rs as well)
            u16::from(ModMask::LOCK) | u16::from(ModMask::M2),
        ]) {
            conn.grab_button(
                false,
                window,
                event_mask as u16,
                GrabMode::ASYNC,
                GrabMode::SYNC,
                x11rb::NONE,
                x11rb::NONE,
                ButtonIndex::ANY,
                mod_key | mask,
            )?;
        }
    } else {
        // Grab everything
        conn.grab_button(
            false,
            window,
            event_mask as u16,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
            x11rb::NONE,
            x11rb::NONE,
            ButtonIndex::ANY,
            ModMask::ANY,
        )?;
    }

    Ok(())
}
