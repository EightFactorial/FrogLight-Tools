use std::path::{Path, PathBuf};

use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::Version;

/// Trait for files that are downloaded and cached.
pub trait FileTrait: Sized + Send + Sync {
    /// Data needed to get the URL of the file.
    type UrlData;

    /// Get the URL of the file.
    fn get_url(version: &Version, data: &Self::UrlData) -> String;

    /// Get the path to the file.
    fn get_path(version: &Version, cache: &Path) -> PathBuf;

    /// Fetch the file, downloading it if it doesn't exist.
    fn fetch(
        version: &Version,
        cache: &Path,
        data: &Self::UrlData,
        redownload: bool,
        client: &Client,
    ) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send + Sync;
}

/// Fetch a JSON file, downloading it if it doesn't exist.
///
/// # Errors
/// Errors if the file can't be read from the cache, downloaded, or parsed.
pub(super) async fn fetch_json<T: FileTrait + DeserializeOwned>(
    version: &Version,
    cache: &Path,
    data: &T::UrlData,
    redownload: bool,
    client: &Client,
) -> anyhow::Result<T> {
    // If the file exists, try to parse it.
    let path = T::get_path(version, cache);
    if path.exists() && !redownload {
        // If the file is successfully parsed, return it.
        match tokio::fs::read(&path).await.map(|bytes| serde_json::from_slice(&bytes)) {
            Ok(Ok(file)) => return Ok(file),
            Ok(Err(err)) => {
                tracing::warn!("Failed to parse file: {err}");
            }
            Err(err) => {
                tracing::warn!("Failed to read file: {err}");
            }
        }
    } else if let Some(parent) = path.parent() {
        // Create the parent directory if it doesn't exist.
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // Download the file.
    let response = client.get(T::get_url(version, data)).send().await?;
    let bytes = response.bytes().await?;

    // Parse the file and write it to the cache.
    let file = serde_json::from_slice(bytes.as_ref())?;
    tokio::fs::write(&path, bytes).await?;

    Ok(file)
}

/// Fetch a YAML file, downloading it if it doesn't exist.
///
/// # Errors
/// Errors if the file can't be read from the cache, downloaded, or parsed.
#[expect(dead_code)]
pub(super) async fn fetch_yaml<T: FileTrait + DeserializeOwned>(
    version: &Version,
    cache: &Path,
    data: &T::UrlData,
    redownload: bool,
    client: &Client,
) -> anyhow::Result<T> {
    // If the file exists, try to parse it.
    let path = T::get_path(version, cache);
    if path.exists() && !redownload {
        // If the file is successfully parsed, return it.
        match tokio::fs::read(&path).await.map(|bytes| serde_yml::from_slice(&bytes)) {
            Ok(Ok(file)) => return Ok(file),
            Ok(Err(err)) => {
                tracing::warn!("Failed to parse file: {err}");
            }
            Err(err) => {
                tracing::warn!("Failed to read file: {err}");
            }
        }
    } else if let Some(parent) = path.parent() {
        // Create the parent directory if it doesn't exist.
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // Download the file.
    let response = client.get(T::get_url(version, data)).send().await?;
    let bytes = response.bytes().await?;

    // Parse the file and write it to the cache.
    let file = serde_yml::from_slice(bytes.as_ref())?;
    tokio::fs::write(&path, bytes).await?;

    Ok(file)
}
