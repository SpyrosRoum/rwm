mod command;
mod errors;
mod tag_id;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub use command::*;
pub use errors::*;
pub use tag_id::TagID;

/// A function that serialises a message and produces a string that can be directly sent to the wm or the client
pub fn into_message<T: Serialize>(msg: T) -> serde_json::Result<String> {
    let json = serde_json::to_string(&msg)?;
    Ok(format!("{:0>4}{}", json.len(), json))
}

#[derive(Deserialize, Serialize, StructOpt, Debug, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
}
