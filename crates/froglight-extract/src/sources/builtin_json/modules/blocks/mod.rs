use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts blocks and block data.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize_unit_struct, Serialize_unit_struct,
)]
pub struct Blocks;

impl ExtractModule for Blocks {
    /// Run the extraction process.
    async fn extract<'a>(&'a self, _data: ExtractBundle<'a>) -> anyhow::Result<()> { Ok(()) }
}
