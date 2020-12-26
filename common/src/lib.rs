mod command;
mod errors;
mod tag_id;

use std::{io::Read, os::unix::net::UnixStream};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub use command::*;
pub use errors::*;
pub use tag_id::TagID;

/// A function that serialises a message and produces a string that can be directly sent to the wm or the client
pub fn into_message<T: Serialize>(msg: T) -> Result<String> {
    let json = serde_json::to_string(&msg).context("Failed to serialise")?;
    Ok(format!("{:0>4}{}", json.len(), json))
}

/// Read a message from the socket
pub fn read_message(stream: &mut UnixStream) -> Result<String> {
    // First for bytes we read should be the length of the command that follows
    let mut cmd_len = [0; 4];
    stream
        .read_exact(&mut cmd_len)
        .context("There was an error in the wm. Failed to read response.")?;
    // If it can't be parsed to a number we simply don't care about it
    let cmd_len = String::from_utf8(cmd_len.to_vec())
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut handle = stream.take(cmd_len as u64);
    let mut reply = String::with_capacity(cmd_len);
    handle
        .read_to_string(&mut reply)
        .context("Failed to read response.")?;
    Ok(reply)
}

#[derive(Deserialize, Serialize, StructOpt, Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
}
