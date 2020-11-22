use std::str::FromStr;

use crate::errors::TagValueError;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct TagID(pub(crate) u8);

impl TagID {
    pub(crate) fn new(tag: u8) -> Result<Self, TagValueError> {
        // Tags can only be from 1 to 9
        if tag < 1 || tag > 9 {
            Err(TagValueError { tag_num: tag })
        } else {
            Ok(Self(tag))
        }
    }
}

// impl TryFrom<u8> for TagID {
//     type Error = TagValueError;
//
//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         Self::new(value)
//     }
// }

impl From<u8> for TagID {
    fn from(value: u8) -> Self {
        Self::new(value).unwrap()
    }
}

impl FromStr for TagID {
    type Err = TagValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag_num = s.parse::<u8>()?;
        Self::new(tag_num)
    }
}
