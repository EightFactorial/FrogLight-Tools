use std::path::Path;

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// A report of all generated items.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemReport(HashMap<CompactString, ItemReportEntry>);

impl ItemReport {
    /// Create a new [`ItemReport`] from the given item report path.
    #[expect(clippy::missing_errors_doc)]
    pub async fn new(report_path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&tokio::fs::read_to_string(report_path).await?)?)
    }
}

/// A report entry for a specific item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemReportEntry {
    /// The components of the item.
    pub components: HashMap<CompactString, serde_json::Value>,
}
