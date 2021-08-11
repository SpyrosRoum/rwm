use {
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

use crate::Direction;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum MonitorSubcommand {
    /// Focus the monitor in the given direction
    Focus(Direction),
}
