use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub(crate) struct ToCommandError {
    pub(crate) text: String,
}

impl fmt::Display for ToCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid command: {}", self.text)
    }
}

impl std::convert::From<TagValueError> for ToCommandError {
    fn from(e: TagValueError) -> Self {
        Self {
            text: format!("Invalid tag number: {}", e.tag_num),
        }
    }
}

impl Error for ToCommandError {}

#[derive(Debug)]
pub(crate) struct TagValueError {
    pub(crate) tag_num: u8,
}

impl fmt::Display for TagValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tags can be from 1 to 9: {}", self.tag_num)
    }
}

impl std::convert::From<std::num::ParseIntError> for TagValueError {
    fn from(_e: ParseIntError) -> Self {
        Self { tag_num: 0 }
    }
}

impl Error for TagValueError {}
