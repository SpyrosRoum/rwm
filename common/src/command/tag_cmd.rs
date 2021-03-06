use {
    serde::{Deserialize, Serialize},
    structopt::StructOpt,
};

use crate::TagId;

#[derive(Deserialize, Serialize, StructOpt, Debug)]
pub enum TagSubcommand {
    /// Toggle the visibility of a tag. If there is only one tag, its visibility cannot be toggled
    Toggle { tag_id: TagId },
    /// Go to another tag, making all tags except the target invincible
    Switch { tag_id: TagId },
}
