use std::{convert::TryFrom, str::FromStr};

use {
    derive_more::Display,
    serde::{Deserialize, Serialize},
};

use crate::TagValueError;

#[derive(Display, Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TagId(u8);

impl TagId {
    /// Produce a `TagId` from the given number with no bound checks
    pub fn from_int_unchecked<N: Into<u8>>(n: N) -> Self {
        Self(n.into())
    }
}

impl FromStr for TagId {
    type Err = TagValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tag_num = s.parse::<u8>()?;
        if !(1..=9).contains(&tag_num) {
            Err(TagValueError { tag_num })
        } else {
            Ok(Self(tag_num))
        }
    }
}

impl TryFrom<u8> for TagId {
    type Error = TagValueError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=9).contains(&value) {
            Err(TagValueError { tag_num: value })
        } else {
            Ok(Self(value))
        }
    }
}
