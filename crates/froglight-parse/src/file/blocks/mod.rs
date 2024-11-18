//! TODO

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;
mod traits;

/// The blocks file for a specific version.
#[derive(Debug, Clone, PartialEq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionBlocks(Vec<BlockSpecification>);

/// A block specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockSpecification {
    /// The ID of the block.
    pub id: u32,
    /// The name of the block.
    pub name: CompactString,
    /// The display name of the block.
    #[serde(rename = "displayName")]
    pub display_name: CompactString,

    /// The hardness of the block.
    pub hardness: f32,
    /// The resistance of the block.
    pub resistance: f32,

    /// The stack size of the block.
    #[serde(rename = "stackSize")]
    pub stack_size: u32,

    /// Whether the block is diggable.
    pub diggable: bool,
    /// The material of the block.
    pub material: CompactString,

    /// Whether the block is transparent.
    pub transparent: bool,
    /// The amount of light the block emits.
    #[serde(rename = "emitLight")]
    pub emit_light: u8,
    /// The filter light value.
    #[serde(rename = "filterLight")]
    pub filter_light: u8,

    /// The default state ID.
    #[serde(rename = "defaultState")]
    pub default_state: u32,
    /// The minimum state ID.
    #[serde(rename = "minStateId")]
    pub min_state_id: u32,
    /// The maximum state ID.
    #[serde(rename = "maxStateId")]
    pub max_state_id: u32,
    /// The states of the block.
    pub states: Vec<BlockSpecificationState>,

    /// The tools that can harvest the block.
    #[serde(default, rename = "harvestTools", skip_serializing_if = "HashMap::is_empty")]
    pub harvest_tools: HashMap<CompactString, bool>,
    /// The drops of the block.
    pub drops: Vec<u32>,

    /// The bounding box of the block.
    #[serde(rename = "boundingBox")]
    pub bounding_box: CompactString,
}

/// A block state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum BlockSpecificationState {
    /// A state based on a boolean.
    #[serde(rename = "bool")]
    Bool {
        /// The name of the state.
        name: CompactString,
        /// The number of possible values.
        ///
        /// This is always 2.
        num_values: u32,
    },
    /// A state based on an enum.
    #[serde(rename = "enum")]
    Enum {
        /// The name of the state.
        name: CompactString,
        /// The number of possible values.
        num_values: u32,
        /// The values of the state.
        ///
        /// Must be the same length as `num_values`.
        values: Vec<CompactString>,
    },
    /// A state based on an integer.
    #[serde(rename = "int")]
    Int {
        /// The name of the state.
        name: CompactString,
        /// The number of possible values.
        num_values: u32,
        /// The values of the state.
        ///
        /// Must be the same length as `num_values`.
        values: Vec<CompactString>,
    },
}

impl BlockSpecificationState {
    /// Returns the name of the state.
    #[must_use]
    pub const fn name(&self) -> &CompactString {
        match self {
            Self::Bool { name, .. } | Self::Enum { name, .. } | Self::Int { name, .. } => name,
        }
    }

    /// Returns the number of possible values for the state.
    #[must_use]
    pub const fn num_values(&self) -> u32 {
        match self {
            Self::Bool { num_values, .. }
            | Self::Enum { num_values, .. }
            | Self::Int { num_values, .. } => *num_values,
        }
    }
}
