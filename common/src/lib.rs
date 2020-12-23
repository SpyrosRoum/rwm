mod command;
mod errors;
mod server_reply;
mod tag_id;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub use command::*;
pub use errors::*;
pub use server_reply::Reply;
pub use tag_id::TagID;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum Direction {
    Up,
    Down,
}
