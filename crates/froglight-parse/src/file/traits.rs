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
    ) -> impl std::future::Future<Output = anyhow::Result<Self>>;
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
    let path = fetch_file::<T>(version, cache, data, redownload, client).await?;
    let contents = tokio::fs::read(&path).await?;
    serde_json::from_slice(&contents).map_err(Into::into)
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
    let path = fetch_file::<T>(version, cache, data, redownload, client).await?;
    let contents = tokio::fs::read(&path).await?;
    serde_yml::from_slice(&contents).map_err(Into::into)
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
    let path = fetch_file::<T>(version, cache, data, redownload, client).await?;
    let contents = tokio::fs::read_to_string(&path).await?;
    quick_xml::de::from_str(&contents).map_err(Into::into)
}

/// Fetch a file, downloading it if it doesn't exist.
pub(super) async fn fetch_file<T: FileTrait>(
    version: &Version,
    cache: &Path,
    data: &T::UrlData,
    redownload: bool,
    client: &Client,
) -> anyhow::Result<PathBuf> {
    // Get the path to the file.
    let path = T::get_path(version, cache);

    // If the file exists and we don't want to redownload it, return early.
    if path.exists() && !redownload {
        return Ok(path);
    }

    // Create the parent directory if it doesn't exist.
    if let Some(parent) = path.parent() {
        // Create the parent directory if it doesn't exist.
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    // Get the URL of the file.
    let url = T::get_url(version, data);
    tracing::warn!("Downloading: \"{url}\"");

    // Download the file.
    let response = client.get(url).send().await?;
    let bytes = response.bytes().await?;
    tokio::fs::write(&path, &bytes).await?;

    Ok(path)
}
