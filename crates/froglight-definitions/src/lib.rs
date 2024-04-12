#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod version;
pub use version::MinecraftVersion;

mod asset_manifest;
// pub use asset_manifest::AssetManifest;

mod releases_manifest;
pub use releases_manifest::{ReleasesLatest, ReleasesManifest, ReleasesManifestEntry};

mod version_manifest;
// pub use version_manifest::VersionManifest;
