mod errors;
mod tag_id;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub use errors::*;
pub use tag_id::TagID;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
#[structopt(name = "rwmc", about = "The rwm client")]
pub enum Command {
    #[structopt(alias = "exit")]
    /// Exit the window manager
    Quit,
    /// Commands related to tags
    Tag(TagSubcommand),
    #[structopt(alias = "win")]
    /// Commands related to the currently focused window
    Window(WindowSubcommand),
    /// Commands related to layouts
    Layout(LayoutSubcommand),
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum WindowSubcommand {
    #[structopt(alias = "kill")]
    /// Kill the current window
    Destroy,
    /// Send the current window to another tag
    Send { tag_id: TagID },
    /// Shift focus from the current window
    Focus(Direction),
    #[structopt(alias = "move")]
    /// Shift the current window up or down
    Shift(Direction),
    /// Toggle an option about the current window
    Toggle(WindowToggle),
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum TagSubcommand {
    /// Toggle the visibility of a tag. If there is only one tag, its visibility cannot be toggled
    Toggle { tag_id: TagID },
    /// Go to another tag, making all tags except the target invincible
    Switch { tag_id: TagID },
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum WindowToggle {
    /// If the window is floating or not
    Float,
}

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum LayoutSubcommand {
    Next,
    #[structopt(alias = "previous")]
    Prev,
}
