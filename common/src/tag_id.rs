use std::{convert::TryFrom, str::FromStr};

use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::TagValueError;

#[derive(Display, Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TagID(u8);

impl FromStr for TagID {
    type Err = TagValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag_num = s.parse::<u8>()?;
        if tag_num < 1 || tag_num > 9 {
            Err(TagValueError { tag_num })
        } else {
            Ok(Self(tag_num))
        }
    }
}

impl TryFrom<u8> for TagID {
    type Error = TagValueError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 1 || value > 9 {
            Err(TagValueError { tag_num: value })
        } else {
            Ok(Self(value))
        }
    }
}
