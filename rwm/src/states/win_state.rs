use std::collections::HashSet;

use x11rb::protocol::xproto::*;

use crate::{states::TagState, tag_id::TagID};

#[derive(Debug, PartialEq)]
pub struct WinState {
    pub(crate) id: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    /// The tags that this window is on
    pub(crate) tags: HashSet<TagID>,
    /// If true then ignoring when tilling windows
    pub(crate) floating: bool,
}

impl WinState {
    pub(crate) fn new(win: Window, geom: &GetGeometryReply, tags: &[TagState]) -> Self {
        Self {
            id: win,
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: geom.height,
            tags: tags
                .iter()
                .filter(|tag_state| tag_state.visible)
                .map(|tag_state| tag_state.id)
                .collect(),
            floating: false,
        }
    }
}
