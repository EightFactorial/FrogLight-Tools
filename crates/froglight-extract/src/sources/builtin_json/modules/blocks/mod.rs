use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts blocks and block data.
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
    Deserialize_unit_struct,
    Serialize_unit_struct,
)]
pub struct Blocks;

impl ExtractModule for Blocks {
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()> {
        Blocks::block_json(data).await?;
        Blocks::block_bytecode(data).await
    }
}

#[allow(clippy::unused_async)]
impl Blocks {
    /// Extract block data from json.
    async fn block_json(_data: &mut ExtractBundle<'_>) -> anyhow::Result<()> { Ok(()) }

    /// Extract extra block data from bytecode.
    async fn block_bytecode(_data: &mut ExtractBundle<'_>) -> anyhow::Result<()> { Ok(()) }
}
