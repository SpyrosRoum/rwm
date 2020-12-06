use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub struct TagValueError {
    pub tag_num: u8,
}

impl fmt::Display for TagValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Tags can be from 1 to 9: {}", self.tag_num)
    }
}

impl From<ParseIntError> for TagValueError {
    fn from(_e: ParseIntError) -> Self {
        Self { tag_num: 0 }
    }
}

impl Error for TagValueError {}

#[derive(Debug)]
pub struct ToCommandError {
    pub text: String,
}

impl fmt::Display for ToCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid command: {}", self.text)
    }
}

impl From<TagValueError> for ToCommandError {
    fn from(e: TagValueError) -> Self {
        Self {
            text: format!("Invalid tag number: {}", e.tag_num),
        }
    }
}

impl From<DirectionValueError> for ToCommandError {
    fn from(e: DirectionValueError) -> Self {
        Self {
            text: format!("Invalid direction: {}", e.msg),
        }
    }
}

impl Error for ToCommandError {}

#[derive(Debug)]
pub struct DirectionValueError {
    pub msg: String,
}

impl fmt::Display for DirectionValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Directions can be \"up\" and \"down\": {}", self.msg)
    }
}

impl Error for DirectionValueError {}
