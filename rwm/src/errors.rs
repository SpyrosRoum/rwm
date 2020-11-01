use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) struct ToCommandError {
    pub(crate) text: String,
}

impl fmt::Display for ToCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid command: {}", self.text)
    }
}

impl Error for ToCommandError {}
