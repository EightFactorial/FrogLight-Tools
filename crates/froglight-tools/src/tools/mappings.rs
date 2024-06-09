//! Functions for downloading and working with `TinyRemapper` and Yarn mappings.

use std::path::{Path, PathBuf};

use froglight_definitions::manifests::YarnVersion;
use reqwest::Client;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

// --- TinyRemapper ---

const REMAPPER_FILE_NAME: &str = "tiny-remapper.jar";
const REMAPPER_URL: &str =
    "https://maven.fabricmc.net/net/fabricmc/tiny-remapper/0.9.0/tiny-remapper-0.9.0-fat.jar";

/// Download the `TinyRemapper` JAR, if it does not exist in the cache.
pub async fn get_tinyremapper(cache: &Path, client: &Client) -> Option<PathBuf> {
    let remapper_path = cache.join(REMAPPER_FILE_NAME);
    if remapper_path.exists() && remapper_path.is_file() {
        debug!("`TinyRemapper` already downloaded: \"{}\"", remapper_path.display());
        return Some(remapper_path);
    }

    info!("Downloading `TinyRemapper` from: \"{}\"", REMAPPER_URL);
    let response = client.get(REMAPPER_URL).send().await.ok()?;
    let bytes = response.bytes().await.ok()?;
    tokio::fs::write(&remapper_path, &bytes).await.ok()?;

    Some(remapper_path)
}

// --- Mappings ---

const CLIENT_MAPPINGS_JAR_URL_PATTERN: &str =
    "https://maven.fabricmc.net/net/fabricmc/yarn/{BUILD}/yarn-{BUILD}-mergedv2.jar";

const CLIENT_MAPPINGS_JAR_FILE_NAME: &str = "yarn-mergedv2.jar";
const CLIENT_MAPPINGS_FILE_NAME: &str = "mappings/mappings.tiny";

/// Download the `Yarn` mappings for a specific build,
/// if they do not exist in the cache.
///
/// TODO: Use the `zip` crate to extract the mappings,
/// otherwise this only works on linux machines with `unzip`.
pub async fn get_yarn_mappings(
    build: &YarnVersion,
    cache: &Path,
    client: &Client,
) -> Option<PathBuf> {
    // Download the mappings JAR
    let mappings_jar_path = cache.join(CLIENT_MAPPINGS_JAR_FILE_NAME);
    if mappings_jar_path.exists() {
        debug!("`Yarn` already downloaded: \"{}\"", mappings_jar_path.display());
    } else {
        let url = CLIENT_MAPPINGS_JAR_URL_PATTERN.replace("{BUILD}", build.as_ref());

        info!("Downloading `Yarn Mappings` from: \"{url}\"");
        let response = client.get(url).send().await.ok()?;
        let bytes = response.bytes().await.ok()?;
        tokio::fs::write(&mappings_jar_path, &bytes).await.ok()?;
    }

    // Extract the mappings from the JAR
    let mappings_path = cache.join(CLIENT_MAPPINGS_FILE_NAME);
    if mappings_path.exists() {
        debug!("`Yarn Mappings` already extracted: \"{}\"", mappings_path.display());
        Some(mappings_path)
    } else {
        info!("Extracting `Yarn Mappings` from: \"{}\"", mappings_jar_path.display());
        let command = tokio::process::Command::new("unzip")
            .arg(mappings_jar_path)
            .arg(CLIENT_MAPPINGS_FILE_NAME)
            .arg("-d")
            .arg(cache)
            .output()
            .await
            .ok()?;

        if command.status.success() {
            Some(mappings_path)
        } else {
            tokio::io::stderr().write_all(command.stderr.as_slice()).await.ok()?;
            None
        }
    }
}
