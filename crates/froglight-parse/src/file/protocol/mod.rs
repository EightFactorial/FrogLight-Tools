//! TODO

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

mod types;
pub use types::*;

#[cfg(test)]
mod test;
mod traits;

/// The protocol file for a specific version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionProtocol {
    /// The types the protocol uses.
    pub types: ProtocolTypeMap,
    /// The packets the protocol uses.
    #[serde(flatten)]
    pub packets: ProtocolPackets,
}

/// The packets used in the protocol.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtocolPackets(HashMap<CompactString, ProtocolState>);

/// A protocol state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolState {
    /// Packets sent from the server to the client.
    #[serde(rename = "toClient")]
    pub clientbound: ProtocolStatePackets,
    /// Packets sent from the client to the server.
    #[serde(rename = "toServer")]
    pub serverbound: ProtocolStatePackets,
}

/// The packet types used in the protocol state.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
pub struct ProtocolStatePackets {
    types: ProtocolTypeMap,
}
