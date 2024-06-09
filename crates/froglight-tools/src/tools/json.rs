//! Tools for generating JSON data from the `Server` JAR.

use std::path::{Path, PathBuf};

use froglight_definitions::manifests::ReleaseDownloads;
use reqwest::Client;
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::{debug, info};

const SERVER_FILE_NAME: &str = "server.jar";

/// Download the `Server` JAR from the release,
/// if it does not exist in the cache.
pub async fn download_server_jar(
    release: &ReleaseDownloads,
    cache: &Path,
    client: &Client,
) -> Option<PathBuf> {
    let server_path = cache.join(SERVER_FILE_NAME);
    if server_path.exists() && server_path.is_file() {
        debug!("`Server` already downloaded: \"{}\"", server_path.display());
        return Some(server_path);
    }

    info!("Downloading `Server` from: \"{}\"", release.server.url);
    let response = client.get(release.server.url.as_str()).send().await.ok()?;
    let bytes = response.bytes().await.ok()?;
    tokio::fs::write(&server_path, &bytes).await.ok()?;

    Some(server_path)
}

const GENERATOR_OUTPUT_FOLDER: &str = "generated";

/// Run the `Server` JAR data generators.
pub async fn generate_server_json(server_jar: &Path, cache: &Path) -> Option<PathBuf> {
    let output_path = cache.join(GENERATOR_OUTPUT_FOLDER);
    let relative_jar_path = server_jar.strip_prefix(cache).ok()?;

    if output_path.exists() && output_path.is_dir() {
        debug!("`Server` JSON already generated: \"{}\"", output_path.display());
        return Some(output_path);
    }

    debug!("Generating JSON data from `Server` JAR:");
    debug!("    Server JAR: \"{}\"", server_jar.display());

    let command = Command::new("java")
        .arg("-DbundlerMainClass=net.minecraft.data.Main")
        .arg("-jar")
        .arg(relative_jar_path)
        .arg("--output")
        .arg(GENERATOR_OUTPUT_FOLDER)
        .arg("--all")
        .current_dir(cache)
        .output()
        .await
        .ok()?;

    if command.status.success() {
        Some(output_path)
    } else {
        tokio::io::stderr().write_all(command.stderr.as_slice()).await.ok()?;
        None
    }
}
