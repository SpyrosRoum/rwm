//! Represents the commands that can be sent from the client to the server
mod config_cmd;
mod layout_cmd;
mod tag_cmd;
mod window_cmd;

use {
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

pub use {
    config_cmd::ConfigSubcommand,
    layout_cmd::LayoutSubcommand,
    tag_cmd::TagSubcommand,
    window_cmd::{WindowSubcommand, WindowToggle},
};

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
    /// Print or load a config
    Config(ConfigSubcommand),
}
