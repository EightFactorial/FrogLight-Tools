//! A bundle of data

use std::path::Path;

use async_zip::tokio::read::fs::ZipFileReader;
use froglight_definitions::{
    manifests::{AssetManifest, ReleaseManifest, VersionManifest, YarnManifest},
    MinecraftVersion,
};

use crate::bytecode::JarContainer;

/// A bundle of data that is passed around between
/// the various data extractor modules.
pub struct ExtractBundle<'a> {
    /// The version of Minecraft that data is being extracted from.
    pub version: &'a MinecraftVersion,
    /// All of the parsed class files.
    pub jar_container: &'a JarContainer,
    /// The ZIP file reader.
    pub jar_reader: &'a ZipFileReader,
    /// All the manifests.
    pub manifests: ManifestBundle<'a>,
    /// The output JSON object that data is being written to.
    pub output: &'a mut serde_json::Value,
    /// The path to the cache directory.
    pub cache_dir: &'a Path,
}

impl<'a> ExtractBundle<'a> {
    /// Create a new [`ExtractBundle`].
    #[must_use]
    pub fn new(
        version: &'a MinecraftVersion,
        jar_container: &'a JarContainer,
        jar_reader: &'a ZipFileReader,
        manifests: ManifestBundle<'a>,
        output: &'a mut serde_json::Value,
        cache_dir: &'a Path,
    ) -> Self {
        Self { version, jar_container, jar_reader, manifests, output, cache_dir }
    }
}

/// A bundle of manifests.
#[derive(Debug, Clone, Copy)]
pub struct ManifestBundle<'a> {
    /// The version manifest.
    pub version: &'a VersionManifest,
    /// The yarn manifest.
    pub yarn: &'a YarnManifest,
    /// The release manifest.
    pub release: &'a ReleaseManifest,
    /// The asset manifest.
    pub asset: &'a AssetManifest,
}

impl<'a> ManifestBundle<'a> {
    /// Create a new [`ManifestBundle`].
    #[must_use]
    pub fn new(
        version: &'a VersionManifest,
        yarn: &'a YarnManifest,
        release: &'a ReleaseManifest,
        asset: &'a AssetManifest,
    ) -> Self {
        Self { version, yarn, release, asset }
    }
}
