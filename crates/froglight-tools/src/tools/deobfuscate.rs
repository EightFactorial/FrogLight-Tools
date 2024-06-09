//! Functions for downloading and working with Enigma

use std::path::{Path, PathBuf};

use froglight_definitions::manifests::ReleaseDownloads;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

const CLIENT_FILE_NAME: &str = "client.jar";

/// Download the `Client` JAR from the release,
/// if it does not exist in the cache.
pub async fn download_client_jar(
    release: &ReleaseDownloads,
    cache: &Path,
    client: &Client,
) -> Option<PathBuf> {
    let client_path = cache.join(CLIENT_FILE_NAME);
    if client_path.exists() && client_path.is_file() {
        debug!("`Client` already downloaded: \"{}\"", client_path.display());
        return Some(client_path);
    }

    info!("Downloading `Client` from: \"{}\"", release.client.url);
    let response = client.get(release.client.url.as_str()).send().await.ok()?;
    let bytes = response.bytes().await.ok()?;
    tokio::fs::write(&client_path, &bytes).await.ok()?;

    Some(client_path)
}

const CLIENT_MAPPED_FILE_NAME: &str = "client-mapped.jar";

/// Deobfuscate the `Client` JAR with the remapper and the mappings,
/// if it does not exist in the cache.
pub async fn deobfuscate_client_jar(
    remapper: &Path,
    client_jar: &Path,
    mappings: &Path,
    cache: &Path,
) -> Option<PathBuf> {
    let mapped_path = cache.join(CLIENT_MAPPED_FILE_NAME);
    if mapped_path.exists() && mapped_path.is_file() {
        debug!("`Client` already exists: \"{}\"", mapped_path.display());
        return Some(mapped_path);
    }

    debug!("Mapping `Client`:");
    debug!("    Remapper: \"{}\"", remapper.display());
    debug!("    Client JAR: \"{}\"", client_jar.display());
    debug!("    Mappings: \"{}\"", mappings.display());
    debug!("    Output: \"{}\"", mapped_path.display());

    info!("Mapping `Client` with `TinyRemapper`");
    let command = tokio::process::Command::new("java")
        .arg("-jar")
        // tinyremapper
        .arg(remapper)
        // input jar
        .arg(client_jar)
        // output jar
        .arg(&mapped_path)
        // mappings file
        .arg(mappings)
        // current mappings
        .arg("official")
        // target mappings
        .arg("named")
        .output()
        .await
        .ok()?;

    if command.status.success() {
        Some(mapped_path)
    } else {
        tokio::io::stderr().write_all(command.stderr.as_slice()).await.ok()?;
        None
    }
}
