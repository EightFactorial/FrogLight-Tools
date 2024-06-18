//! Data extracted from parsing Minecraft's built-in json generators.

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

mod modules;
pub use modules::*;

/// Modules that parse Minecraft's built-in json generators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[enum_dispatch(ExtractModule)]
#[serde(untagged)]
pub enum BuiltinJsonModule {
    /// Debug information
    Debug(Debug),
    /// The `version.json` file
    Version(Version),
    /// Blocks and block data
    Blocks(Blocks),
    /// Registries and registry data
    Registries(Registries),
    /// Tags and tag data
    Tags(Tags),
}
