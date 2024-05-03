use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A placeholder module.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize_unit_struct, Deserialize_unit_struct,
)]
pub struct Placeholder;

impl ExtractModule for Placeholder {
    /// Run the extraction process.
    async fn extract<'a>(&'a self, _: ExtractBundle<'a>) -> anyhow::Result<()> { Ok(()) }
}
