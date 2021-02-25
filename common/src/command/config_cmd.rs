use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum ConfigSubcommand {
    /// Print the current configuration
    Print,
    /// (Re)load config from a file
    Load {
        /// Path to the configuration file. If left empty it will try to reload the last file used.
        path: Option<PathBuf>,
    },
}
