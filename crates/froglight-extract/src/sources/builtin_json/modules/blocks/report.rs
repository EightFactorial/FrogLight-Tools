use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// A block report generated by the `Server` jar.
///
/// Contains all blockstates, their ids, and their properties.
///
/// Typically located at `generated/reports/blocks.json`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BlocksReport(pub HashMap<String, BlockData>);

impl std::ops::Deref for BlocksReport {
    type Target = HashMap<String, BlockData>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for BlocksReport {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

/// Block states and properties of a specific block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockData {
    /// A list of all blockstates.
    pub states: Vec<BlockState>,

    /// A map of all blockstate properties and their possible values.
    #[serde(alias = "properties")]
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub state_properties: HashMap<String, Vec<String>>,
}

/// A blockstate with its id and properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockState {
    /// The blockstate id.
    ///
    /// This is the id stored in chunks and send over the network.
    pub id: u32,

    /// The blockstate properties.
    ///
    /// All properties inside the parent [`BlockData`] must be present.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
}