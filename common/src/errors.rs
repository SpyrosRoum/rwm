use std::{error::Error, fmt, io, num::ParseIntError};

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
pub struct ParseModMaskError {
    pub mask: String,
}

impl fmt::Display for ParseModMaskError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid mod mask: {}", self.mask)
    }
}

impl Error for ParseModMaskError {}

#[derive(Debug)]
pub struct ParseColorError {
    pub color: String,
}

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid color: {}", self.color)
    }
}

impl From<ParseIntError> for ParseColorError {
    fn from(e: ParseIntError) -> Self {
        Self {
            color: e.to_string(),
        }
    }
}

impl Error for ParseColorError {}

#[derive(Debug)]
pub struct LoadConfigError {
    pub error: String,
}

impl LoadConfigError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { error: msg.into() }
    }
}

impl From<toml::de::Error> for LoadConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::new(e.to_string())
    }
}

impl fmt::Display for LoadConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to load configuration: {}", self.error)
    }
}

impl From<io::Error> for LoadConfigError {
    fn from(_: io::Error) -> Self {
        Self {
            error: "Error parsing the configuration file".to_string(),
        }
    }
}

impl Error for LoadConfigError {}
