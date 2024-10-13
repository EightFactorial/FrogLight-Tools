use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, consts::GIT_HASH, sources::ExtractModule};

/// A module that adds debug information to the output.
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
pub struct Debug;

impl ExtractModule for Debug {
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()> {
        data.output["debug"]["build-date"] = env!("VERGEN_BUILD_DATE").into();
        data.output["debug"]["git-hash"] = GIT_HASH.into();

        // data.output["debug"]["git-dirty"] = env!("VERGEN_GIT_DIRTY").into();
        data.output["debug"]["git-dirty"] = "unknown".into();

        data.output["debug"]["git-branch"] = env!("VERGEN_GIT_BRANCH").into();
        data.output["debug"]["version"] = env!("CARGO_PKG_VERSION").into();
        Ok(())
    }
}
