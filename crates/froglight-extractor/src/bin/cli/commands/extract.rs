use froglight_data::VersionManifest;

use super::Command;
use crate::classmap::ClassMap;

pub(crate) async fn extract(command: &Command, manifest: &VersionManifest) -> serde_json::Value {
    let _classmap = ClassMap::new(command, manifest).await;

    todo!()
}
