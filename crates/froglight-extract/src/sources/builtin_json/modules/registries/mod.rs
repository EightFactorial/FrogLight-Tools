use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts registries and registry data.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Deserialize_unit_struct, Serialize_unit_struct,
)]
pub struct Registries;

impl ExtractModule for Registries {
    /// Run the extraction process.
    async fn extract<'a>(&'a self, _data: ExtractBundle<'a>) -> anyhow::Result<()> { Ok(()) }
}
