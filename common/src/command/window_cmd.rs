use {
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

use crate::{Direction, TagID};

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

#[derive(Deserialize, Serialize, StructOpt, Debug, Copy, Clone)]
pub enum WindowToggle {
    /// If the window is floating or not
    Float,
}
