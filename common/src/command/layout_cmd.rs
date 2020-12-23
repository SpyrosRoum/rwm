use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum LayoutSubcommand {
    Next,
    #[structopt(alias = "previous")]
    Prev,
}
