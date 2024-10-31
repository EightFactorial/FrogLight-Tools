use anyhow::{anyhow, bail};
use async_zip::tokio::read::fs::ZipFileReader;
use froglight_extract::{
    bundle::{ExtractBundle, ManifestBundle},
    bytecode::JarContainer,
    sources::ExtractModule,
};
use serde_json::{Map, Value};
use tracing::{debug, error, info};

use crate::cli::ExtractArguments;

/// Extract data from the specified version.
#[allow(clippy::too_many_lines)]
pub(super) async fn extract(args: &ExtractArguments) -> anyhow::Result<Value> {
    // Create a `Client` for downloading files
    let client = reqwest::Client::new();

    // --- Prepare Manifests ---

    // Get the `VersionManifest`
    let version_manifest =
        froglight_tools::manifests::get_version_manifest(&args.cache, &client).await?;

    // Get the `YarnManifest`
    let yarn_manifest = froglight_tools::manifests::get_yarn_manifest(&args.cache, &client).await?;

    // Get the version entry
    let Some(version_entry) = version_manifest.get(&args.version).cloned() else {
        error!("Version not found: {}", args.version);
        error!("Try emptying the cache directory and trying again.");
        return Err(anyhow!("Version not found in `VersionManifest`"));
    };
    info!("Found Version in Manifest: \"{}\"", args.version);
    debug!("Version Entry: {version_entry:#?}");

    // Append the version to the cache path
    let mut cache = args.cache.clone();
    cache.push(version_entry.id.to_short_string());
    if !cache.exists() {
        if let Err(err) = tokio::fs::create_dir_all(&cache).await {
            error!("Failed to create cache directory: {err}");
            return Err(err.into());
        }
    }

    // Get the `ReleaseManifest`
    let release_manifest =
        froglight_tools::manifests::get_release_manifest(&version_entry, &cache, &client).await?;
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

    // Get `TinyRemapper`
    let Some(remapper_path) =
        froglight_tools::mappings::get_tinyremapper(&args.cache, &client).await
    else {
        bail!("Failed to download `TinyRemapper` JAR");
    };

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

    // --- Extract  ---

    // Create a `ManifestBundle`
    let manifest_bundle = ManifestBundle::new(
        version_manifest.into(),
        yarn_manifest.into(),
        release_manifest,
        asset_manifest,
    );

    // Create an `ExtractBundle`
    let mut extract_bundle = ExtractBundle::new(
        version_entry.id.clone(),
        jar_container,
        jar_reader,
        manifest_bundle,
        Value::Object(Map::new()),
        cache,
        json_path,
    );

    // Sort modules and extract data
    let mut modules = args.modules.clone();
    modules.sort();
    modules.dedup();

    info!("Extracting data for: \"{}\"", version_entry.id);
    debug!("    Modules: {:?}", modules);
    for module in modules {
        if let Err(err) = module.extract(&mut extract_bundle).await {
            error!("Error running `{module:?}`: {err}");
        }
    }
    info!("Done!");

    // Return the extracted data
    Ok(extract_bundle.output)
}
