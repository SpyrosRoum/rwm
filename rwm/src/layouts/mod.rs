mod grid;
mod monad_tall;

use x11rb::{errors::ReplyOrIdError, rust_connection::RustConnection};

use crate::focus_history::FocusHist;
use common::TagID;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum LayoutType {
    MonadTall,
    Grid,
    Floating,
}

impl LayoutType {
    pub(crate) fn update(
        &self,
        conn: &RustConnection,
        focus: &FocusHist,
        tags: Vec<TagID>,
        screen_num: usize,
        border_width: u32,
    ) -> Result<(), ReplyOrIdError> {
        match self {
            LayoutType::MonadTall => {
                monad_tall::update(conn, focus, tags, screen_num, border_width)?
            }
            LayoutType::Floating => {} // We don't have anything to do
            LayoutType::Grid => grid::update(conn, focus, tags, screen_num, border_width)?,
        };

        Ok(())
    }

    /// Find the next layout in the list
    pub(crate) fn next(&self, layouts: &[Self]) -> Self {
        let cur_pos = layouts.iter().position(|cur| cur == self).unwrap();
        layouts.get(cur_pos + 1).unwrap_or(&layouts[0]).to_owned()
    }

    /// Find the previous layout in the list
    pub(crate) fn prev(&self, layouts: &[Self]) -> Self {
        let cur_pos = layouts.iter().position(|cur| cur == self).unwrap();
        let new = cur_pos.checked_sub(1).unwrap_or(layouts.len() - 1);
        layouts[new].to_owned()
    }
}
