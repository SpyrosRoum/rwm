use std::convert::TryInto;

use crate::layouts::LayoutType;
use common::{TagID, TagValueError};

#[derive(Debug, Copy, Clone)]
pub(crate) struct TagState {
    pub(crate) id: TagID,
    pub(crate) visible: bool,
    pub(crate) layout: LayoutType,
}

impl PartialEq for TagState {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq<TagID> for TagState {
    fn eq(&self, other: &TagID) -> bool {
        &self.id == other
    }
}

impl TagState {
    pub(crate) fn new(tag: u8, visible: bool, layout: LayoutType) -> Result<Self, TagValueError> {
        Ok(Self {
            id: tag.try_into()?,
            visible,
            layout,
        })
    }
}
