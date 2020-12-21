//! A struct that represents a reply from rwm to a client
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Reply {
    /// True for success, False for error
    pub status: bool,
    pub message: Option<String>,
}
