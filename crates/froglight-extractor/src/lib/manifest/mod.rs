//! Functions for loading various manifests

mod version;
pub use version::version_manifest;

mod release;
pub use release::release_manifest;

mod asset;
pub use asset::asset_manifest;
