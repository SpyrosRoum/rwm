use {
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum LayoutSubcommand {
    Next,
    #[structopt(alias = "previous")]
    Prev,
}
