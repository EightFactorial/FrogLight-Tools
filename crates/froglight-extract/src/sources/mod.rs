//! Extraction Sources

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

pub mod builtin_json;
#[allow(clippy::wildcard_imports)]
use builtin_json::*;

pub mod bytecode;
#[allow(clippy::wildcard_imports)]
use bytecode::*;

use crate::bundle::ExtractBundle;

/// Modules that extract data from various sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[enum_dispatch(ExtractModule)]
#[serde(untagged)]
pub enum Modules {
    /// Data extracted from parsing Minecraft's built-in json generators.
    BuiltinJson(BuiltinJsonModule),
    /// Data extracted from bytecode parsing.
    Bytecode(BytecodeModule),
}

/// Trait for extracting data from a source.
#[enum_dispatch]
pub trait ExtractModule {
    /// Run the extraction process.
    #[allow(async_fn_in_trait)]
    async fn extract<'a>(&'a self, data: ExtractBundle<'a>) -> anyhow::Result<()>;
}
