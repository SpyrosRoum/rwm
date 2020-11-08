use std::str::FromStr;

use clap::{App, AppSettings, Arg, SubCommand};

use crate::errors::ToCommandError;
use crate::newtypes::Tag;

#[derive(Debug)]
pub(crate) enum TagSubCommand {
    Toggle(Tag),
    Switch(Tag),
}

#[derive(Debug)]
pub(crate) enum WindowSubcommand {
    Destroy,
}

#[derive(Debug)]
pub(crate) enum Command {
    Quit,
    Tag(TagSubCommand),
    Window(WindowSubcommand),
}

impl FromStr for Command {
    type Err = ToCommandError;

    fn from_str(cmd_str: &str) -> Result<Self, Self::Err> {
        let cmd_str = cmd_str.to_ascii_lowercase();

        let cmd_parts = cmd_str.split_ascii_whitespace().collect::<Vec<_>>();
        if cmd_parts.is_empty() {
            return Err(ToCommandError { text: cmd_str });
        }

        let command = App::new("Command")
            .setting(AppSettings::NoBinaryName)
            .subcommand(SubCommand::with_name("quit"))
            .subcommand(
                SubCommand::with_name("tag")
                    .subcommand(SubCommand::with_name("switch").arg(Arg::with_name("tag").index(1)))
                    .subcommand(
                        SubCommand::with_name("toggle").arg(Arg::with_name("tag").index(1)),
                    ),
            )
            .subcommand(
                SubCommand::with_name("window")
                    .subcommand(SubCommand::with_name("destroy").alias("kill")),
            )
            .get_matches_from_safe(&cmd_parts);

        if command.is_err() {
            return Err(ToCommandError { text: cmd_str });
        }
        let command = command.unwrap();

        match command.subcommand() {
            ("quit", _) => Ok(Self::Quit),
            ("tag", Some(subcommands)) => match subcommands.subcommand() {
                ("toggle", Some(args)) => {
                    if args.value_of("tag").is_none() {
                        return Err(ToCommandError { text: cmd_str });
                    }
                    let tag = Tag::from_str(args.value_of("tag").unwrap())?;
                    Ok(Self::Tag(TagSubCommand::Toggle(tag)))
                }
                ("switch", Some(args)) => {
                    if args.value_of("tag").is_none() {
                        return Err(ToCommandError { text: cmd_str });
                    }
                    let tag = Tag::from_str(args.value_of("tag").unwrap())?;
                    Ok(Self::Tag(TagSubCommand::Switch(tag)))
                }
                _ => Err(ToCommandError { text: cmd_str }),
            },
            ("window", Some(subcommands)) => match subcommands.subcommand() {
                ("destroy", _) => Ok(Self::Window(WindowSubcommand::Destroy)),
                _ => Err(ToCommandError { text: cmd_str }),
            },
            _ => Err(ToCommandError { text: cmd_str }),
        }
    }
}
