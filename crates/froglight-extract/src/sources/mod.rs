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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()>;
}

/// Implement `FromStr` for `Modules` to allow parsing from a string.
mod json_workaround {
    use std::str::FromStr;

    use serde::{Deserialize, Serialize};

    use super::Modules;

    #[derive(Serialize, Deserialize)]
    struct ModuleWrapper {
        module: Modules,
    }

    impl FromStr for Modules {
        type Err = anyhow::Error;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match serde_json::from_str::<ModuleWrapper>(&format!("{{\"module\": \"{s}\"}}")) {
                Ok(resolver) => Ok(resolver.module),
                Err(err) => Err(anyhow::anyhow!("Failed to parse module: {err}")),
            }
        }
    }
}
