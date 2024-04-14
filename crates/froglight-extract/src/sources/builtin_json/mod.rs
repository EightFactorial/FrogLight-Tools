//! Data extracted from parsing Minecraft's built-in json generators.

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

mod modules;
pub use modules::*;

/// Json modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[enum_dispatch(ExtractModule)]
#[serde(untagged)]
pub enum BuiltinJsonModule {
    /// Blocks and block data
    Blocks(Blocks),
    /// Registries and registry data
    Registries(Registries),
}
