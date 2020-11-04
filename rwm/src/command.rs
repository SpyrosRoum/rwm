use std::str::FromStr;

use crate::errors::ToCommandError;
use crate::newtypes::Tag;

#[derive(Debug)]
pub(crate) enum TagSubCommand {
    Toggle(Tag),
    Switch(Tag),
}

#[derive(Debug)]
pub(crate) enum Command {
    Quit,
    Tag(TagSubCommand),
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
            return Ok(Self::Quit);
        } else if cmd_parts[0] == "tag" {
            if let Some(sub_command) = cmd_parts.get(1) {
                if sub_command == &"toggle" || sub_command == &"switch" {
                    if let Some(tag) = cmd_parts.get(2) {
                        let tag = Tag::from_str(tag)?;
                        return if sub_command == &"toggle" {
                            Ok(Self::Tag(TagSubCommand::Toggle(tag)))
                        } else {
                            Ok(Self::Tag(TagSubCommand::Switch(tag)))
                        };
                    }
                }
            }
        }
        Err(ToCommandError { text: cmd_str })
    }
}
