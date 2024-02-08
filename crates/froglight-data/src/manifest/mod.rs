//! Data manifests and their structures.

pub(super) mod version_manifest;
pub use version_manifest::{ManifestLatest, ManifestVersion};

pub(super) mod release_manifest;
pub use release_manifest::{ReleaseAssetIndex, ReleaseDownload, ReleaseDownloads};

pub(super) mod asset_manifest;
pub use asset_manifest::AssetObject;
