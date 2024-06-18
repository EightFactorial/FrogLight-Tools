use std::{path::PathBuf, sync::Arc};

use anyhow::bail;
use async_zip::tokio::read::fs::ZipFileReader;
use froglight_definitions::manifests::{VersionManifest, YarnManifest};
use froglight_extract::{
    bundle::{ExtractBundle, ManifestBundle},
    bytecode::JarContainer,
    sources::{ExtractModule, Modules as ExtractModules},
};
use froglight_generate::{
    bundle::{GenerateBundle, GenerateVersion},
    modules::{GenerateModule, Modules},
};
use reqwest::Client;
use serde_json::{Map, Value};
use tracing::{debug, error, info};

/// Generate code for a specific version.
#[allow(clippy::too_many_lines)]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn generate(
    version: GenerateVersion,
    modules: Vec<Modules>,

    version_manifest: Arc<VersionManifest>,
    yarn_manifest: Arc<YarnManifest>,
    remapper_path: PathBuf,

    mut cache: PathBuf,
    root_dir: PathBuf,
    client: Client,
) -> anyhow::Result<()> {
    // --- Prepare Manifests ---

    // Get the version entry
    let Some(version_entry) = version_manifest.get(&version.jar) else {
        error!("Version not found: {}", version.jar);
        error!("Try emptying the cache directory and trying again.");
        bail!("Version not found in `VersionManifest`");
    };
    info!("Found Version in Manifest: \"{}\"", version.jar);
    debug!("Version Entry: {version_entry:#?}");

    // Append the version to the cache path
    cache.push(version_entry.id.to_short_string());
    if !cache.exists() {
        tokio::fs::create_dir_all(&cache).await?;
    }

    // Get the `ReleaseManifest`
    let release_manifest =
        froglight_tools::manifests::get_release_manifest(version_entry, &cache, &client).await?;
    info!("Loaded Release Manifest for: \"{}\"", version_entry.id);

    // Get the `AssetManifest`
    let asset_manifest =
        froglight_tools::manifests::get_asset_manifest(&release_manifest, &cache, &client).await?;

    info!("Loaded Asset Manifest for: \"{}\"", version_entry.id);
    debug!("Asset Manifest: {} Assets", asset_manifest.objects.len());

    // --- Run `Server` JAR Generators ---

    // Download the `Server` JAR
    let Some(server_jar) =
        froglight_tools::json::download_server_jar(&release_manifest.downloads, &cache, &client)
            .await
    else {
        bail!("Failed to download `Server` JAR");
    };
    debug!("`Server` JAR: \"{}\"", server_jar.display());

    // Run the `Server` JAR generators
    info!("Running \"{}\" `Server` JAR generators ...", version_entry.id);
    let Some(json_path) = froglight_tools::json::generate_server_json(&server_jar, &cache).await
    else {
        bail!("Failed to generate `Server` JSON");
    };
    info!("Generated \"{}\" `Server` JSON files", version_entry.id);

    // --- Parse `Client` JAR ---

    // Download the `Client` JAR
    let Some(client_jar) = froglight_tools::deobfuscate::download_client_jar(
        &release_manifest.downloads,
        &cache,
        &client,
    )
    .await
    else {
        bail!("Failed to download `Client` JAR");
    };
    debug!("`Client` JAR: \"{}\"", client_jar.display());

    // Get the latest Yarn mappings for this version
    let Some(yarn_build) = yarn_manifest.get_latest(&version_entry.id) else {
        bail!("No Yarn mappings found for: \"{}\"", version_entry.id);
    };
    info!("Using Yarn: \"{yarn_build}\"");

    let Some(yarn_mappings) =
        froglight_tools::mappings::get_yarn_mappings(yarn_build, &cache, &client).await
    else {
        bail!("Failed to download `Yarn` mappings");
    };

    // Get the deobfuscated `Client` JAR
    let Some(mapped_jar) = froglight_tools::deobfuscate::deobfuscate_client_jar(
        &remapper_path,
        &client_jar,
        &yarn_mappings,
        &cache,
    )
    .await
    else {
        bail!("Failed to deobfuscate `Client` JAR");
    };

    // Read and parse the deobfuscated `Client` JAR
    info!("Parsing \"{}\" `Client` JAR ...", version_entry.id);
    let jar_reader = ZipFileReader::new(mapped_jar).await?;
    let jar_container = JarContainer::new_tokio_fs(&jar_reader).await?;
    info!("Successfully parsed \"{}\" `Client` JAR", version_entry.id);
    debug!("\"{}\" parsed {} classes", version_entry.id, jar_container.len());

    // --- Extract and Generate ---

    // Create a `Value` to store extracted data
    let mut extract_data = Value::Object(Map::new());

    // Create a `ManifestBundle`
    let manifest_bundle =
        ManifestBundle::new(&version_manifest, &yarn_manifest, &release_manifest, &asset_manifest);

    // Create an `ExtractBundle`
    let mut extract_bundle = ExtractBundle::new(
        &version_entry.id,
        &jar_container,
        &jar_reader,
        manifest_bundle,
        &mut extract_data,
        &cache,
        &json_path,
    );

    // Extract data
    {
        // Get the required extract modules and run them
        let mut extract_modules: Vec<ExtractModules> = Vec::new();
        for generate_module in &modules {
            extract_modules.extend(generate_module.required());
        }
        extract_modules.sort();
        extract_modules.dedup();

        info!("Extracting data for: \"{}\"", version_entry.id);
        debug!("    Extract Modules: {extract_modules:?}");
        for module in extract_modules {
            module.extract(&mut extract_bundle).await?;
        }
    }

    // Generate code
    {
        // Create a `GenerateBundle`
        let generate_bundle = GenerateBundle::new(&version, &root_dir);

        info!("Generating code for: \"{}\"", version_entry.id);
        debug!("    Generate Modules: {modules:?}");
        for module in &modules {
            module.generate(&generate_bundle, &extract_bundle).await?;
        }
    }

    Ok(())
}
