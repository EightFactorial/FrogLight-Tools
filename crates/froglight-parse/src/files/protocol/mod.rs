//! TODO

use serde::{Deserialize, Serialize};

mod types;
pub use types::*;

/// The protocol file for a specific version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionProtocol {
    /// The types the protocol uses.
    pub types: TypesMap,
}

impl VersionProtocol {
    /// The name of the protocol file.
    pub const FILE_NAME: &str = "protocol.json";
}
