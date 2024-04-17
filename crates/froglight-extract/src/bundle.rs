//! A bundle of data

use std::path::Path;

use froglight_definitions::{
    manifests::{AssetManifest, ReleaseManifest, VersionManifest},
    MinecraftVersion,
};

/// A bundle of data that is passed around between
/// the various data extractor modules.
#[derive(Debug)]
pub struct ExtractBundle<'a> {
    /// All the manifests.
    pub manifests: ManifestBundle<'a>,
    /// The version of Minecraft that data is being extracted from.
    pub version: &'a MinecraftVersion,
    /// The output JSON object that data is being written to.
    pub output: &'a mut serde_json::Value,
    /// The path to the cache directory.
    pub cache_dir: &'a Path,
}

/// A bundle of manifests.
#[derive(Debug, Clone, Copy)]
pub struct ManifestBundle<'a> {
    /// The version manifest.
    pub version: &'a VersionManifest,
    /// The release manifest.
    pub release: &'a ReleaseManifest,
    /// The asset manifest.
    pub asset: &'a AssetManifest,
}