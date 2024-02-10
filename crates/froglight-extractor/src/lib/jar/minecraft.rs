use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use froglight_data::{Version, VersionManifest};
use tracing::{debug, info, trace};

use crate::manifest;

/// Get the mapped jar from the cache or map it using `TinyRemapper`
///
/// # Errors
/// - If the cache directory cannot be created
/// - If the mapper fails to download
/// - If the mappings fail to download
/// - If the jar fails to download
/// - If the mapper fails to run
pub async fn get_mapped_jar(
    version: &Version,
    manifest: &VersionManifest,
    cache: &Path,
    refresh: bool,
) -> anyhow::Result<PathBuf> {
    let mapper = super::get_mapper(cache, refresh).await?;
    let mappings = super::get_mappings(version, cache, refresh).await?;
    let jar = get_jar(version, manifest, cache, refresh).await?;

    create_mapped_jar(refresh, &mapper, &mappings, &jar).await
}

/// Get the jar from the cache or download it from the server
///
/// # Errors
/// - If the cache directory cannot be created
/// - If the jar fails to download
/// - If the jar fails to write to the cache
pub async fn get_jar(
    version: &Version,
    manifest: &VersionManifest,
    cache: &Path,
    refresh: bool,
) -> anyhow::Result<PathBuf> {
    let mut jar_path = cache.join("froglight");
    jar_path.push(version.to_short_string());

    // Create the cache directory if it doesn't exist
    if !jar_path.exists() {
        debug!("Creating version cache directory: {}", jar_path.display());
        tokio::fs::create_dir_all(&jar_path).await?;
    }

    jar_path.push("client.jar");

    trace!("JarPath: {}", jar_path.display());

    // Check if the jar is already downloaded
    if refresh || !jar_path.exists() {
        // Get the release manifest
        let version =
            manifest.get(version).ok_or(anyhow::anyhow!("Version not found in VersionManifest"))?;
        let release = manifest::release_manifest(version, cache, refresh).await?;

        info!("Downloading client.jar...");
        trace!("ClientJar URL: {}", release.downloads.client.url);

        // Download the jar
        let response = reqwest::get(&release.downloads.client.url).await?;

        // Save the jar to the cache
        let bytes = response.bytes().await?;
        tokio::fs::write(&jar_path, &bytes).await?;
    }

    Ok(jar_path)
}

/// Map the jar using `TinyRemapper`
///
/// # Panics
/// - If `TinyRemapper` fails to run
async fn create_mapped_jar(
    refresh: bool,
    mapper: &PathBuf,
    mappings: &PathBuf,
    jar: &Path,
) -> anyhow::Result<PathBuf> {
    let mut mapped_jar = jar.to_path_buf();
    mapped_jar.set_file_name("client_mapped.jar");

    if refresh || !mapped_jar.exists() {
        info!("Running TinyRemapper...");

        tokio::process::Command::new("java")
            .arg("-jar")
            .arg(mapper)
            .arg(jar)
            .arg(&mapped_jar)
            .arg(mappings)
            .arg("official")
            .arg("named")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?
            .wait()
            .await?;
    }

    Ok(mapped_jar)
}
