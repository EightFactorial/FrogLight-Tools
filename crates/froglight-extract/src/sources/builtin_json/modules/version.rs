use anyhow::bail;
use serde_json::Value;
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts the `version.json` file.
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
pub struct Version;

impl ExtractModule for Version {
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()> {
        let reader = data.jar_reader;

        // Find the `version.json` entry in the JAR.
        let Some((entry_index, _entry)) = reader
            .file()
            .entries()
            .iter()
            .enumerate()
            .find(|(_, entry)| matches!(entry.filename().as_str(), Ok("version.json")))
        else {
            bail!("\"version.json\" not found in JAR");
        };

        // Get the `version.json` entry and write it to a buffer.
        let mut entry = reader.reader_with_entry(entry_index).await?;

        let mut buffer = String::new();
        entry.read_to_string_checked(&mut buffer).await?;

        // Parse the buffer into a JSON object.
        let version_json: Value = serde_json::from_str(&buffer)?;
        data.output["version"] = version_json;

        Ok(())
    }
}
