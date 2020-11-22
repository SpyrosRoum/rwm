use std::{error::Error, fs, str::FromStr};

use x11rb::protocol::xproto::KeyButMask;

use crate::{direction::Direction, errors::ToCommandError, states::TagState, tag_id::TagID};

pub(crate) fn clean_mask(mask: u16) -> u16 {
    // TODO: num lock is not always Mod2, find a way to get that dynamically
    mask & !(KeyButMask::Mod2 | KeyButMask::Lock)
        & (KeyButMask::Shift
            | KeyButMask::Control
            | KeyButMask::Mod1
            | KeyButMask::Mod2
            | KeyButMask::Mod3
            | KeyButMask::Mod4
            | KeyButMask::Mod5)
}

pub(crate) fn clean_up() -> Result<(), Box<dyn Error>> {
    fs::remove_file("/tmp/rwm.sock")?;

    Ok(())
}

pub(crate) fn get_tag(args: &clap::ArgMatches) -> Result<TagID, ToCommandError> {
    if args.value_of("tag").is_none() {
        return Err(ToCommandError {
            text: "Missing tag value".to_string(),
        });
    }
    let tag = args.value_of("tag").unwrap();
    let tag = TagID::from_str(tag)?;
    Ok(tag)
}

pub(crate) fn get_direction(args: &clap::ArgMatches) -> Result<Direction, ToCommandError> {
    if args.value_of("direction").is_none() {
        return Err(ToCommandError {
            text: "Missing direction".to_string(),
        });
    }
    let direction = args.value_of("direction").unwrap();
    let direction = Direction::from_str(direction)?;
    Ok(direction)
}

/// Get all the visible tags only
pub(crate) fn visible(tags: &[TagState]) -> Vec<TagState> {
    tags.iter().filter(|tag| tag.visible).copied().collect()
}
