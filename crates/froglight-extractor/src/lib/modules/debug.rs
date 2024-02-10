use std::path::Path;

use froglight_data::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Extract;
use crate::classmap::ClassMap;

/// A module that appends debug information to the output.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct DebugModule;

impl Extract for DebugModule {
    async fn extract(
        &self,
        version: &Version,
        classmap: &ClassMap,
        _: &Path,
        output: &mut Value,
    ) -> anyhow::Result<()> {
        // Add debug information to the output
        output["debug"] = serde_json::json!({
            "build": env!("VERGEN_GIT_SHA"),
            "build_date": env!("VERGEN_BUILD_DATE"),
            "dirty": env!("VERGEN_GIT_DIRTY"),
            "target": version.to_string(),
            "classes": classmap.len(),
        });

        Ok(())
    }
}
