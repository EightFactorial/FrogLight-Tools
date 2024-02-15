use std::path::Path;

use async_zip::tokio::read::fs::ZipFileReader;
use froglight_data::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::trace;

use super::Extract;
use crate::classmap::ClassMap;

/// A module that extracts the version's `version.json` file.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct InfoModule;

impl Extract for InfoModule {
    async fn extract(
        &self,
        version: &Version,
        _: &ClassMap,
        cache: &Path,
        output: &mut Value,
    ) -> anyhow::Result<()> {
        let mut jar_path = cache.to_path_buf();
        jar_path.push("froglight");
        jar_path.push(version.to_short_string());
        jar_path.push("client_mapped.jar");

        trace!("Reading version.json from {jar_path:?}");

        let jar = ZipFileReader::new(jar_path).await?;
        let file_count = jar.file().entries().len();

        for index in 0..file_count {
            let mut entry = jar.reader_with_entry(index).await?;
            let Ok(filename) = entry.entry().filename().as_str() else {
                continue;
            };

            if filename == "version.json" {
                let mut contents = String::new();
                entry.read_to_string_checked(&mut contents).await?;

                let json: Value = serde_json::from_str(&contents)?;
                output["version"] = json;
            }
        }

        Ok(())
    }
}
