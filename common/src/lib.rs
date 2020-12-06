mod errors;
mod tag_id;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub use errors::*;
pub use tag_id::TagID;

// TODO write doc strings for these
#[derive(Deserialize, Serialize, StructOpt, Debug)]
#[structopt(name = "rwmc", about = "The rwm client")]
pub enum Command {
    #[structopt(alias = "exit")]
    /// Exit the window manager
    Quit,
    Tag(TagSubcommand),
    #[structopt(alias = "win")]
    Window(WindowSubcommand),
    Layout(LayoutSubcommand),
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum WindowSubcommand {
    #[structopt(alias = "kill")]
    Destroy,
    Send {
        tag_id: TagID,
    },
    Focus(Direction),
    Toggle(WindowToggle),
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum TagSubcommand {
    Toggle { tag_id: TagID },
    Switch { tag_id: TagID },
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum WindowToggle {
    Float,
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum LayoutSubcommand {
    Next,
    #[structopt(alias = "previous")]
    Prev,
}
