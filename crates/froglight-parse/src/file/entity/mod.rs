//! TODO

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;
mod traits;

/// The entity file for a specific version.
#[derive(Debug, Clone, PartialEq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VersionEntities(Vec<EntitySpecification>);

/// An entity specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitySpecification {
    /// The entity ID.
    pub id: u32,
    /// The entity internal ID.
    #[serde(rename = "internalId")]
    pub internal_id: u32,

    /// The entity name.
    pub name: CompactString,
    /// The entity display name.
    #[serde(rename = "displayName")]
    pub display_name: CompactString,

    /// The entity width.
    pub width: f32,
    /// The entity height.
    pub height: f32,

    /// The entity kind.
    #[serde(rename = "type")]
    pub kind: CompactString,
    /// The entity category.
    pub category: CompactString,

    /// The entity metadata.
    #[serde(rename = "metadataKeys")]
    pub metadata: Vec<CompactString>,
}
