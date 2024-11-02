use std::path::Path;

use compact_str::CompactString;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// A packet report for a specific [`Version`](crate::Version).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketReport {
    /// The configuration connection state.
    pub configuration: PacketReportState,
    /// The handshake connection state.
    pub handshake: PacketReportState,
    /// The login connection state.
    pub login: PacketReportState,
    /// The play connection state.
    pub play: PacketReportState,
    /// The status connection state.
    pub status: PacketReportState,

    /// Other connection states.
    #[serde(flatten)]
    pub other: HashMap<CompactString, PacketReportState>,
}

impl PacketReport {
    /// Create a new [`PacketReport`] from the given packets report path.
    #[expect(clippy::missing_errors_doc)]
    pub async fn new(report_path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&tokio::fs::read_to_string(report_path).await?)?)
    }
}

/// A connection state in a [`PacketReport`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketReportState {
    /// Packets sent from the server to the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clientbound: Option<PacketReportDirection>,
    /// Packets sent from the client to the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serverbound: Option<PacketReportDirection>,
}

/// A direction in a [`PacketReportState`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PacketReportDirection(HashMap<CompactString, PacketReportPacket>);

/// A packet in a [`PacketReport`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketReportPacket {
    /// The packet ID.
    pub protocol_id: u32,
}
