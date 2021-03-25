use serde::{Deserialize, Serialize};

use common::TagId;

/// A simple spawn rule for new windows
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SpawnRule {
    // match against WM_CLASS
    ClassName(String, Vec<TagId>),
    // match against WM_NAME
    WmName(String, Vec<TagId>),
}
