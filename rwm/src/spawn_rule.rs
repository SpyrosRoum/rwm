use serde::{Deserialize, Serialize};

use common::TagID;

/// A simple spawn rule for new windows
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum SpawnRule {
    // match against WM_CLASS
    ClassName(String, TagID),
    // match against WM_NAME
    WMName(String, TagID),
}
