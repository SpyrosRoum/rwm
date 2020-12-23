use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum ConfigSubcommand {
    /// Print the current configuration
    Print,
    Load {
        /// Path to the configuration file. If left empty it will try to reload the current file.
        path: Option<PathBuf>,
    },
}
