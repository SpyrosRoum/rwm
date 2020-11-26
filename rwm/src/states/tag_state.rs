use crate::{errors::TagValueError, layouts::LayoutType, tag_id::TagID};

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
        // Tags can only be from 1 to 9
        if tag < 1 || tag > 9 {
            Err(TagValueError { tag_num: tag })
        } else {
            Ok(Self {
                id: tag.into(),
                visible,
                layout,
            })
        }
    }
}
