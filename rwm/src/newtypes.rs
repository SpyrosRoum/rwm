use std::str::FromStr;

use crate::errors::TagValueError;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
pub(crate) struct Tag(u8);

impl Tag {
    pub(crate) fn new(tag: u8) -> Result<Self, TagValueError> {
        // Tags can only be from 1 to 9
        if tag < 1 || tag > 9 {
            Err(TagValueError { tag_num: tag })
        } else {
            Ok(Tag(tag))
        }
    }
}

impl FromStr for Tag {
    type Err = TagValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag_num = s.parse::<u8>()?;
        Self::new(tag_num)
    }
}
