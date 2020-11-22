use crate::{errors::TagValueError, tag_id::TagID};

#[derive(Debug, Copy, Clone)]
pub(crate) struct TagState {
    pub(crate) id: TagID,
    pub(crate) visible: bool,
}

impl From<TagID> for TagState {
    // We only care about the tag number since this is what's used for comparisons
    fn from(tag: TagID) -> Self {
        Self {
            id: tag,
            visible: false,
        }
    }
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
    pub(crate) fn new(tag: u8, visible: bool) -> Result<Self, TagValueError> {
        // Tags can only be from 1 to 9
        if tag < 1 || tag > 9 {
            Err(TagValueError { tag_num: tag })
        } else {
            Ok(Self {
                id: tag.into(),
                visible,
            })
        }
    }
}
