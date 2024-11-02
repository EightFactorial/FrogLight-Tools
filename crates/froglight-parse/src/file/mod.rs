//! TODO

mod datapath;
pub use datapath::{DataPath, EditionDataPath, VersionDataPath};

pub mod protocol;
pub use protocol::VersionProtocol;

mod traits;
use traits::fetch_json;
pub use traits::FileTrait;

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
