//! TODO

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::Version;

#[cfg(test)]
mod test;
mod traits;

/// Data paths for Minecraft version data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataPath {
    /// [`EditionDataPath`] for the Java edition.
    pub pc: EditionDataPath,
    /// [`EditionDataPath`] for the Bedrock edition.
    pub bedrock: EditionDataPath,
}

impl DataPath {
    /// The name of the data paths file.
    pub const FILE_NAME: &str = "dataPaths.json";
    /// The URL of the data paths file.
    pub const FILE_URL: &str = "https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/dataPaths.json";

    /// Get the URL for a Java edition `proto.yml` file.
    #[must_use]
    pub fn get_java_proto(&self, version: &Version) -> Option<String> {
        let proto = self.pc.get(version).and_then(|paths| paths.proto.as_ref())?;
        Some(Self::FILE_URL.replace("dataPaths.json", proto) + "/proto.yml")
    }

    /// Get the URL for a Java edition `protocol.json` file.
    #[must_use]
    pub fn get_java_protocol(&self, version: &Version) -> Option<String> {
        let protocol = self.pc.get(version).and_then(|paths| paths.protocol.as_ref())?;
        Some(Self::FILE_URL.replace("dataPaths.json", protocol) + "/protocol.json")
    }
}

/// Data paths for a specific edition.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EditionDataPath(HashMap<Version, VersionDataPath>);

/// Data paths for a specific version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct VersionDataPath {
    #[serde(default)]
    pub attributes: Option<CompactString>,
    #[serde(default)]
    pub blocks: Option<CompactString>,
    #[serde(default, rename = "blockCollisionShapes")]
    pub block_collision_shapes: Option<CompactString>,
    #[serde(default)]
    pub biomes: Option<CompactString>,
    #[serde(default)]
    pub effects: Option<CompactString>,
    #[serde(default)]
    pub items: Option<CompactString>,
    #[serde(default)]
    pub enchantments: Option<CompactString>,
    #[serde(default)]
    pub recipes: Option<CompactString>,
    #[serde(default)]
    pub instruments: Option<CompactString>,
    #[serde(default)]
    pub materials: Option<CompactString>,
    #[serde(default)]
    pub language: Option<CompactString>,
    #[serde(default)]
    pub entities: Option<CompactString>,
    #[serde(default)]
    pub protocol: Option<CompactString>,
    #[serde(default)]
    pub windows: Option<CompactString>,
    #[serde(default)]
    pub version: Option<CompactString>,
    #[serde(default)]
    pub foods: Option<CompactString>,
    #[serde(default)]
    pub particles: Option<CompactString>,
    #[serde(default, rename = "blockLoot")]
    pub block_loot: Option<CompactString>,
    #[serde(default, rename = "entityLoot")]
    pub entity_loot: Option<CompactString>,
    #[serde(default, rename = "loginPacket")]
    pub login_packet: Option<CompactString>,
    #[serde(default)]
    pub tints: Option<CompactString>,
    #[serde(rename = "mapIcons")]
    #[serde(default)]
    pub map_icons: Option<CompactString>,
    #[serde(default)]
    pub commands: Option<CompactString>,
    #[serde(default)]
    pub sounds: Option<CompactString>,
    #[serde(default)]
    pub proto: Option<CompactString>,

    #[serde(default, flatten)]
    pub other: HashMap<CompactString, CompactString>,
}
