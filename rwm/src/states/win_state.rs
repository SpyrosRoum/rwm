use crate::newtypes::Tag;
use std::collections::HashSet;
use x11rb::protocol::xproto::*;

#[derive(Debug, PartialEq)]
pub struct WinState {
    pub(crate) id: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    /// The tags that this window is on
    pub(crate) tags: HashSet<Tag>,
}

impl WinState {
    pub(crate) fn new(win: Window, geom: &GetGeometryReply, tags: HashSet<Tag>) -> Self {
        Self {
            id: win,
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: geom.height,
            tags,
        }
    }
}
