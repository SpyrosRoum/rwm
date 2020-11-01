use std::str::FromStr;

use crate::errors::ToCommandError;

#[derive(Debug)]
pub(crate) enum Command {
    Quit,
}

impl FromStr for Command {
    type Err = ToCommandError;

    fn from_str(cmd_str: &str) -> Result<Self, Self::Err> {
        let cmd_str = cmd_str.to_ascii_lowercase();

        let cmd_parts = cmd_str.split_ascii_whitespace().collect::<Vec<_>>();
        if cmd_parts.is_empty() {
            return Err(ToCommandError { text: cmd_str });
        }

        if cmd_parts[0] == "quit" {
            Ok(Self::Quit)
        } else {
            Err(ToCommandError { text: cmd_str })
        }
    }
}
