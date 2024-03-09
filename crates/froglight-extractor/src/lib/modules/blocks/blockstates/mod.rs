use std::path::Path;

use froglight_data::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{classmap::ClassMap, modules::Extract};

/// A module to extract block states.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BlockStateModule;

impl Extract for BlockStateModule {
    async fn extract(
        &self,
        _: &Version,
        _classmap: &ClassMap,
        _: &Path,
        _output: &mut Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
