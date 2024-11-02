use std::path::Path;

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// A report of all generated blocks.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegistryReport(HashMap<CompactString, RegistryReportEntries>);

impl RegistryReport {
    /// Create a new [`RegistryReport`] from the given registry report path.
    #[expect(clippy::missing_errors_doc)]
    pub async fn new(report_path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&tokio::fs::read_to_string(report_path).await?)?)
    }
}

/// A map of registry entries for a specific registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryReportEntries {
    /// The default registry entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<CompactString>,
    /// The registry entries.
    pub entries: HashMap<CompactString, RegistryEntry>,
}

/// A registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// The ID of the entry.
    pub protocol_id: u32,
}
