//! Functions for working with Mojang's manifests.

use std::path::Path;

use froglight_definitions::manifests::{
    AssetManifest, ReleaseManifest, VersionManifest, VersionManifestEntry, YarnManifest,
};
use reqwest::Client;
use tracing::{debug, info};

/// An error that occurred while working with a manifest.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    /// An IO error occurred.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A [`serde_json`] error occurred.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// A [`quick_xml`] error occurred.
    #[error(transparent)]
    Xml(#[from] quick_xml::de::DeError),
    /// A [`reqwest`] error occurred.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

// --- Version Manifest ---

const VERSION_MANIFEST_NAME: &str = "version_manifest_v2.json";

/// Read the [`VersionManifest`] from the cache or download it.
///
/// # Errors
/// Returns an error if the manifest could not be read, parsed, or downloaded.
pub async fn get_version_manifest(
    cache: &Path,
    client: &Client,
) -> Result<VersionManifest, ManifestError> {
    let manifest_path = cache.join(VERSION_MANIFEST_NAME);
    if manifest_path.exists() && manifest_path.is_file() {
        debug!("Loading `VersionManifest` from cache: \"{}\"", manifest_path.display());
        let content = tokio::fs::read_to_string(manifest_path).await?;
        serde_json::from_str(&content).map_err(ManifestError::Json)
    } else {
        download_version_manifest(cache, client).await
    }
}

/// Download the [`VersionManifest`] from Mojang.
///
/// # Errors
/// Returns an error if the manifest could not be downloaded or parsed.
pub async fn download_version_manifest(
    cache: &Path,
    client: &Client,
) -> Result<VersionManifest, ManifestError> {
    debug!("Downloading `VersionManifest` from: \"{}\"", VersionManifest::URL);

    // Download the [`VersionManifest`]
    let response = client.get(VersionManifest::URL).send().await?;
    let content = response.text().await?;

    // Write the manifest to the cache
    let manifest_path = cache.join(VERSION_MANIFEST_NAME);
    info!("Caching `VersionManifest` at: \"{}\"", manifest_path.display());
    tokio::fs::write(&manifest_path, &content).await?;

    // Parse and return the manifest
    serde_json::from_str(&content).map_err(ManifestError::Json)
}

// --- Yarn Manifest ---

const YARN_MANIFEST_NAME: &str = "yarn-mavex-metadata.xml";

/// Read the [`YarnManifest`] from the cache or download it.
///
/// # Errors
/// Returns an error if the manifest could not be read, parsed, or downloaded.
pub async fn get_yarn_manifest(
    cache: &Path,
    client: &Client,
) -> Result<YarnManifest, ManifestError> {
    let manifest_path = cache.join(YARN_MANIFEST_NAME);
    if manifest_path.exists() && manifest_path.is_file() {
        debug!("Loading `YarnManifest` from cache: \"{}\"", manifest_path.display());
        let content = tokio::fs::read_to_string(manifest_path).await?;
        quick_xml::de::from_str(&content).map_err(ManifestError::Xml)
    } else {
        download_yarn_manifest(cache, client).await
    }
}

/// Download the [`YarnManifest`] from the Fabric Maven.
///
/// # Errors
/// Returns an error if the manifest could not be downloaded or parsed.
pub async fn download_yarn_manifest(
    cache: &Path,
    client: &Client,
) -> Result<YarnManifest, ManifestError> {
    debug!("Downloading `YarnManifest` from: \"{}\"", YarnManifest::URL);

    // Download the [`YarnManifest`]
    let response = client.get(YarnManifest::URL).send().await?;
    let content = response.text().await?;

    // Write the manifest to the cache
    let manifest_path = cache.join(YARN_MANIFEST_NAME);
    info!("Caching `YarnManifest` at: \"{}\"", manifest_path.display());
    tokio::fs::write(&manifest_path, &content).await?;

    // Parse and return the manifest
    quick_xml::de::from_str(&content).map_err(ManifestError::Xml)
}

// --- Release Manifest ---

/// Read the [`ReleaseManifest`] from the cache or download it.
///
/// # Errors
/// Returns an error if the manifest could not be read, parsed, or downloaded.
pub async fn get_release_manifest(
    version: &VersionManifestEntry,
    cache: &Path,
    client: &Client,
) -> Result<ReleaseManifest, ManifestError> {
    let manifest_path = cache.join(format!("{}.json", version.id.to_short_string()));
    if manifest_path.exists() && manifest_path.is_file() {
        debug!("Loading `ReleaseManifest` from cache: \"{}\"", manifest_path.display());
        let content = tokio::fs::read_to_string(manifest_path).await?;
        serde_json::from_str(&content).map_err(ManifestError::Json)
    } else {
        download_release_manifest(version, cache, client).await
    }
}

/// Download the [`ReleaseManifest`] from Mojang.
///
/// # Errors
/// Returns an error if the manifest could not be downloaded or parsed.
pub async fn download_release_manifest(
    version: &VersionManifestEntry,
    cache: &Path,
    client: &Client,
) -> Result<ReleaseManifest, ManifestError> {
    debug!("Downloading `ReleaseManifest` from: \"{}\"", version.url);
    let response = client.get(version.url.as_str()).send().await?;
    let content = response.text().await?;

    let manifest_path = cache.join(format!("{}.json", version.id.to_short_string()));
    info!("Caching `ReleaseManifest` at: \"{}\"", manifest_path.display());
    tokio::fs::write(&manifest_path, &content).await?;

    serde_json::from_str(&content).map_err(ManifestError::Json)
}

// --- Asset Manifest ---

const ASSET_MANIFEST_NAME: &str = "asset_index.json";

/// Read the [`AssetManifest`] from the cache or download it.
///
/// # Errors
/// Returns an error if the manifest could not be read, parsed, or downloaded.
pub async fn get_asset_manifest(
    release: &ReleaseManifest,
    cache: &Path,
    client: &Client,
) -> Result<AssetManifest, ManifestError> {
    let manifest_path = cache.join(ASSET_MANIFEST_NAME);
    if manifest_path.exists() && manifest_path.is_file() {
        debug!("Loading `AssetManifest` from cache: \"{}\"", manifest_path.display());
        let content = tokio::fs::read_to_string(manifest_path).await?;

        serde_json::from_str(&content).map_err(ManifestError::Json)
    } else {
        download_asset_manifest(release, cache, client).await
    }
}

/// Download the [`AssetManifest`] from Mojang.
///
/// # Errors
/// Returns an error if the manifest could not be downloaded or parsed.
pub async fn download_asset_manifest(
    release: &ReleaseManifest,
    cache: &Path,
    client: &Client,
) -> Result<AssetManifest, ManifestError> {
    debug!("Downloading `AssetManifest` from: \"{}\"", release.asset_index.url);
    let response = client.get(release.asset_index.url.as_str()).send().await?;
    let content = response.text().await?;

    let manifest_path = cache.join(ASSET_MANIFEST_NAME);
    info!("Caching `AssetManifest` at: \"{}\"", manifest_path.display());
    tokio::fs::write(&manifest_path, &content).await?;

    serde_json::from_str(&content).map_err(ManifestError::Json)
}
