//! TODO

use std::path::Path;

mod block;
pub use block::*;

mod item;
pub use item::*;

mod packet;
pub use packet::*;

mod registry;
pub use registry::*;

/// Reports generated for a specific [`Version`](crate::Version).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedReports {
    /// Generated blocks.
    pub blocks: BlockReport,
    /// Generated items.
    pub items: ItemReport,
    /// Generated packets.
    pub packets: PacketReport,
    /// Generated registries.
    pub registries: RegistryReport,
}

impl GeneratedReports {
    /// Create a new [`GeneratedReports`] from the given reports path.
    #[expect(clippy::missing_errors_doc)]
    pub async fn new(reports_path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            blocks: BlockReport::new(&reports_path.join("blocks.json")).await?,
            items: ItemReport::new(&reports_path.join("items.json")).await?,
            packets: PacketReport::new(&reports_path.join("packets.json")).await?,
            registries: RegistryReport::new(&reports_path.join("registries.json")).await?,
        })
    }
}
