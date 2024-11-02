use std::path::Path;

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// A report of all generated blocks.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockReport(HashMap<CompactString, GeneratedBlock>);

impl BlockReport {
    /// Create a new [`BlocksReport`] from the given blocks report path.
    #[expect(clippy::missing_errors_doc)]
    pub async fn new(report_path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&tokio::fs::read_to_string(report_path).await?)?)
    }
}

/// A generated block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneratedBlock {
    /// The block definition.
    pub definition: BlockDefinition,
    /// The block properties.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<CompactString, Vec<CompactString>>,
    /// The block states.
    pub states: Vec<BlockState>,
}

/// A block definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockDefinition {
    /// The type of the block.
    #[serde(rename = "type")]
    pub kind: CompactString,
    /// The properties of the block.
    pub properties: HashMap<CompactString, serde_json::Value>,
    /// Other block definition fields.
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<CompactString, serde_json::Value>,
}

/// A block state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockState {
    /// Whether this is the default state.
    #[serde(default, skip_serializing_if = "BlockState::is_false")]
    pub default: bool,
    /// The ID of the block state.
    pub id: u32,
    /// The properties of the block state.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<CompactString, CompactString>,
}

impl BlockState {
    #[expect(clippy::trivially_copy_pass_by_ref)]
    fn is_false(b: &bool) -> bool { !*b }
}
