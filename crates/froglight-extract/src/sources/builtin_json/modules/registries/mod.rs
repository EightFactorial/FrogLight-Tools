use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts registries and registry data.
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
pub struct Registries;

impl ExtractModule for Registries {
    async fn extract<'a>(&self, _data: &mut ExtractBundle<'a>) -> anyhow::Result<()> { Ok(()) }
}
