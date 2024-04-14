//! Data extracted from bytecode parsing.

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

mod modules;
pub use modules::*;

/// Bytecode modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[enum_dispatch(ExtractModule)]
#[serde(untagged)]
pub enum BytecodeModule {
    /// Placeholder module.
    Placeholder(Placeholder),
}
