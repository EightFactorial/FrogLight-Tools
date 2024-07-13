use std::{future::Future, pin::Pin};

use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        builtin_json::{Blocks as ExtractBlocks, BuiltinJsonModule},
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use super::sealed::GenerateRequired;
use crate::{bundle::GenerateBundle, modules::GenerateModule};

// mod generated;
// mod version;

/// A module that generates blocks and registries.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize_unit_struct,
    Deserialize_unit_struct,
)]
pub struct Blocks;

impl Blocks {
    /// The path to the `froglight-block` src folder,
    /// relative to the root directory.
    const _CRATE_SRC_PATH: &'static str = "crates/froglight-block/src";
}

impl GenerateRequired for Blocks {
    const REQUIRED: &'static [ExtractModules] =
        &[ExtractModules::BuiltinJson(BuiltinJsonModule::Blocks(ExtractBlocks))];
}

impl GenerateModule for Blocks {
    /// Run the generation process.
    fn generate<'a>(
        &'a self,
        _generate: &'a GenerateBundle<'_>,
        _extract: &'a ExtractBundle<'_>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + Sync + 'a>> {
        Box::pin(async { todo!() })
    }
}
