//! A bundle of data

use std::{path::PathBuf, sync::Arc};

use async_zip::tokio::read::fs::ZipFileReader;
use froglight_definitions::{
    manifests::{AssetManifest, ReleaseManifest, VersionManifest, YarnManifest},
    MinecraftVersion,
};

use crate::bytecode::JarContainer;

/// A bundle of data that is passed around between
/// the various data extractor modules.
pub struct ExtractBundle {
    /// The version of Minecraft that data is being extracted from.
    pub version: MinecraftVersion,
    /// All of the parsed class files.
    pub jar_container: JarContainer,
    /// The ZIP file reader.
    pub jar_reader: ZipFileReader,
    /// All the manifests.
    pub manifests: ManifestBundle,
    /// The output JSON object that data is being written to.
    pub output: serde_json::Value,
    /// The path to the cache directory.
    pub cache_dir: PathBuf,
    /// The path to the json directory.
    pub json_dir: PathBuf,
}

impl ExtractBundle {
    /// Create a new [`ExtractBundle`].
    #[must_use]
    pub fn new(
        version: MinecraftVersion,
        jar_container: JarContainer,
        jar_reader: ZipFileReader,
        manifests: ManifestBundle,
        output: serde_json::Value,
        cache_dir: PathBuf,
        json_dir: PathBuf,
    ) -> Self {
        Self { version, jar_container, jar_reader, manifests, output, cache_dir, json_dir }
    }
}

/// A bundle of manifests.
#[derive(Debug, Clone)]
pub struct ManifestBundle {
    /// The version manifest.
    pub version: Arc<VersionManifest>,
    /// The yarn manifest.
    pub yarn: Arc<YarnManifest>,
    /// The release manifest.
    pub release: ReleaseManifest,
    /// The asset manifest.
    pub asset: AssetManifest,
}

impl ManifestBundle {
    /// Create a new [`ManifestBundle`].
    #[must_use]
    pub fn new(
        version: Arc<VersionManifest>,
        yarn: Arc<YarnManifest>,
        release: ReleaseManifest,
        asset: AssetManifest,
    ) -> Self {
        Self { version, yarn, release, asset }
    }
}
