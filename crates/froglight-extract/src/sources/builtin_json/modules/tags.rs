use std::ffi::OsStr;

use anyhow::bail;
use serde_json::{Map, Value};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use walkdir::WalkDir;

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that extracts tags from the `tags` directory.
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
pub struct Tags;

impl ExtractModule for Tags {
    async fn extract<'a>(&self, data: &mut ExtractBundle<'a>) -> anyhow::Result<()> {
        let tag_dir = data.json_dir.join("data").join("minecraft").join("tags");
        if !tag_dir.exists() || !tag_dir.is_dir() {
            bail!("\"tags\" directory not found in JSON directory");
        }

        let mut tags = Value::Object(Map::new());

        for entry in WalkDir::new(&tag_dir) {
            let entry = entry?;

            // Get the relative path of the entry, skipping if it's not a file
            let mut entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }
            entry_path = entry_path.strip_prefix(&tag_dir)?;

            // Get the map to insert the tag into, creating one if it doesn't exist
            let mut map = tags.as_object_mut().unwrap();
            for part in entry_path.parent().unwrap().iter().map(|p| p.to_str().unwrap()) {
                map = map.entry(part).or_insert(Value::Object(Map::new())).as_object_mut().unwrap();
            }

            // Prefix the tag name with "minecraft:" and insert it into the map
            let tag_name = entry_path
                .file_name()
                .and_then(OsStr::to_str)
                .map(|s| format!("minecraft:{}", s.trim_end_matches(".json")))
                .unwrap();
            map.insert(
                tag_name,
                serde_json::from_str(&tokio::fs::read_to_string(entry.path()).await?)?,
            );
        }

        data.output["tags"] = tags;

        Ok(())
    }
}
