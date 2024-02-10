use std::path::{Path, PathBuf};

use froglight_data::Version;
use tracing::{debug, info, trace};

const MAPPER_VERSION: &str = "0.9.0";
const MAPPER_URL: &str = "https://maven.fabricmc.net/net/fabricmc/tiny-remapper/{MAP_VER}/tiny-remapper-{MAP_VER}-fat.jar";
const MAPPER_FILE: &str = "tiny-remapper.jar";

/// Get the `TinyRemapper` from the cache or download it from the server
///
/// # Errors
/// - If the `TinyRemapper` fails to download
/// - If the `TinyRemapper` fails to write to the cache
pub async fn get_mapper(cache: &Path, refresh: bool) -> anyhow::Result<PathBuf> {
    let mut mapper_path = cache.join("froglight");
    mapper_path.push(MAPPER_FILE);

    // Check if the mapper is already downloaded
    if refresh || !mapper_path.exists() {
        // Download the mapper
        let url = MAPPER_URL.replace("{MAP_VER}", MAPPER_VERSION);
        let response = reqwest::get(&url).await?;

        // Save the mapper to the cache
        let bytes = response.bytes().await?;
        tokio::fs::write(&mapper_path, &bytes).await?;
    }

    Ok(mapper_path)
}

/// The URL for the mappings
const MAPPINGS_URL: &str =
    "https://maven.fabricmc.net/net/fabricmc/yarn/{BUILD_VER}/yarn-{BUILD_VER}-mergedv2.jar";
/// The path to the mappings inside the jar
const MAPPINGS_JAR_FILE_PATH: &str = "mappings/mappings.tiny";
/// The name of the mappings file
const MAPPINGS_FILE: &str = "mappings.tiny";

/// Get the mappings from the cache or download them from the server
///
/// # Errors
/// - If the mappings fail to download
/// - If the mappings fail to write to the cache
pub async fn get_mappings(
    version: &Version,
    cache: &Path,
    refresh: bool,
) -> anyhow::Result<PathBuf> {
    let mut mappings_path = cache.join("froglight");
    mappings_path.push(version.to_short_string());

    // Create the cache directory if it doesn't exist
    if !mappings_path.exists() {
        debug!("Creating version cache directory: {}", mappings_path.display());
        tokio::fs::create_dir_all(&mappings_path).await?;
    }

    mappings_path.push(MAPPINGS_FILE);

    trace!("MappingsPath: {}", mappings_path.display());

    // Check if the mappings are already downloaded
    if refresh || !mappings_path.exists() {
        // Get the latest build of the mappings
        let latest = get_latest_build(version, cache, refresh).await?;

        // Download the mappings
        let url = MAPPINGS_URL.replace("{BUILD_VER}", &latest);
        let response = reqwest::get(&url).await?;

        // Open the mappings file as a zip
        let bytes = response.bytes().await?.to_vec();
        let zip = async_zip::base::read::mem::ZipFileReader::new(bytes).await?;

        // Get the mappings file from the zip
        let file_index = zip.file().entries().iter().position(|entry| {
            if let Ok(name) = entry.filename().as_str() {
                name == MAPPINGS_JAR_FILE_PATH
            } else {
                false
            }
        });
        let Some(file_index) = file_index else {
            anyhow::bail!("Mappings file not found in the jar");
        };
        let mut entry = zip.reader_with_entry(file_index).await?;

        // Save the mappings to the cache
        let mut content = Vec::new();
        entry.read_to_end_checked(&mut content).await?;

        tokio::fs::write(&mappings_path, &content).await?;
    }

    Ok(mappings_path)
}

/// The URL for the mappings directory
const MAPPINGS_DIRECTORY_URL: &str = "https://maven.fabricmc.net/net/fabricmc/yarn/";
/// The name of the mappings directory file
const MAPPINGS_DIRECTORY_FILE: &str = "mappings_directory.txt";

// Find the latest build of the mappings
async fn get_latest_build(
    version: &Version,
    cache: &Path,
    refresh: bool,
) -> anyhow::Result<String> {
    let mut directory_path = cache.join("froglight");
    directory_path.push(MAPPINGS_DIRECTORY_FILE);

    let directory;

    // Check if the mappings directory is already downloaded
    if refresh || !directory_path.exists() {
        info!("Downloading mappings directory...");
        trace!("MappingsDirectory URL: {}", MAPPINGS_DIRECTORY_URL);

        // Download the mappings directory
        let response = reqwest::get(MAPPINGS_DIRECTORY_URL).await?;

        let content = response.text().await?;
        tokio::fs::write(&directory_path, &content).await?;

        directory = content;
    } else {
        trace!("Loading mappings directory from cache");

        directory = tokio::fs::read_to_string(&directory_path).await?;
    }

    let mut builds = Vec::new();
    for line in directory.lines() {
        // Find the build inside the tags
        if let Some((_, line)) = line.split_once('>') {
            if let Some((build, _)) = line.split_once('/') {
                // If it's a build for the current version, add it to the list
                let build_version = format!("{}+", version.to_short_string());
                if build.starts_with(&build_version) {
                    builds.push(build.to_string());
                }
            }
        }
    }

    // Sort the builds and get the latest
    builds.sort();

    if let Some(build) = builds.pop() {
        debug!("Using mappings build: `{build}`");
        Ok(build)
    } else {
        anyhow::bail!("No builds found for Version `{}`", version.to_short_string());
    }
}
