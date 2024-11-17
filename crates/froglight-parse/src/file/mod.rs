//! TODO

pub mod blocks;
pub use blocks::VersionBlocks;

pub mod datapath;
pub use datapath::DataPath;

pub mod generator;
pub use generator::GeneratorData;

pub mod manifest;
pub use manifest::VersionManifest;

pub mod yarnmaven;
pub use yarnmaven::YarnMavenMetadata;

pub mod protocol;
pub use protocol::VersionProtocol;

pub mod versioninfo;
pub use versioninfo::VersionInfo;

mod traits;
pub use traits::FileTrait;
use traits::{fetch_file, fetch_json, fetch_xml};

/// Get the `target` directory.
///
/// Used for testing.
#[cfg(test)]
pub(super) async fn target_dir() -> std::path::PathBuf {
    let mut cache = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    while !cache.join("target").exists() {
        assert!(!cache.to_string_lossy().is_empty(), "Couldn't find target directory");
        cache = cache.parent().unwrap().to_path_buf();
    }

    cache.push("target");
    cache.push("froglight-parse");
    tokio::fs::create_dir_all(&cache).await.unwrap();

    cache
}
