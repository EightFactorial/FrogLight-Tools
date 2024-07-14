//! Data extracted from parsing Minecraft's built-in json generators.

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

mod resource_pack;
pub use resource_pack::ResourcePack;

/// Modules that make requests to Mojang's API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[enum_dispatch(ExtractModule)]
#[serde(untagged)]
pub enum MojangModule {
    /// Resource Pack assets
    ResourcePack(ResourcePack),
}
