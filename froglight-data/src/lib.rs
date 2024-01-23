#![doc = include_str!("../README.md")]

pub mod manifest;
pub use manifest::{
    asset_manifest::AssetManifest, release_manifest::ReleaseManifest,
    version_manifest::VersionManifest,
};

mod version;
pub use version::Version;
