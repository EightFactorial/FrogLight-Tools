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
    let url = T::get_url(version, data);
    tracing::warn!("Downloading: \"{url}\"");

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(&path, &bytes).await?;

    // Parse the file
    serde_json::from_slice(&bytes).map_err(Into::into)
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
    let url = T::get_url(version, data);
    tracing::warn!("Downloading: \"{url}\"");

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(&path, &bytes).await?;

    // Parse the file
    serde_yml::from_slice(&bytes).map_err(Into::into)
}

/// Fetch a XML file, downloading it if it doesn't exist.
///
/// # Errors
/// Errors if the file can't be read from the cache, downloaded, or parsed.
pub(super) async fn fetch_xml<T: FileTrait + DeserializeOwned>(
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
        match tokio::fs::read_to_string(&path).await.map(|string| quick_xml::de::from_str(&string))
        {
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
    let url = T::get_url(version, data);
    tracing::warn!("Downloading: \"{url}\"");

    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(&path, &bytes).await?;

    // Parse the file
    let contents = std::str::from_utf8(&bytes)?;
    quick_xml::de::from_str(contents).map_err(Into::into)
}
