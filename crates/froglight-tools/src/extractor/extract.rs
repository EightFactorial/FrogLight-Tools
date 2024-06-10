use anyhow::anyhow;
use async_zip::tokio::read::fs::ZipFileReader;
use froglight_extract::{
    bundle::{ExtractBundle, ManifestBundle},
    bytecode::JarContainer,
    sources::{ExtractModule, Modules},
};
use serde_json::{Map, Value};
use tracing::{debug, error, info};

use crate::cli::ExtractArguments;

/// Extract data from the specified version.
#[allow(clippy::too_many_lines)]
pub(super) async fn extract(args: &ExtractArguments) -> anyhow::Result<Value> {
    // --- Prepare Files ---

    // Create a `Client` for downloading files
    let client = reqwest::Client::new();

    // Get the `VersionManifest`
    let version_manifest =
        froglight_tools::manifests::get_version_manifest(&args.cache, &client).await?;

    // Get the `YarnManifest`
    let yarn_manifest = froglight_tools::manifests::get_yarn_manifest(&args.cache, &client).await?;

    // Get `TinyRemapper`
    let Some(remapper_path) =
        froglight_tools::mappings::get_tinyremapper(&args.cache, &client).await
    else {
        let error = "Failed to download `TinyRemapper` JAR";

        error!("{error}");
        return Err(anyhow!(error));
    };

    // Get the version entry
    let Some(version_entry) = version_manifest.get(&args.version) else {
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
    let release_manifest = match froglight_tools::manifests::get_release_manifest(
        version_entry,
        &cache,
        &client,
    )
    .await
    {
        Ok(manifest) => manifest,
        Err(err) => {
            error!("Failed to get `ReleaseManifest`: {err}");
            return Err(err.into());
        }
    };
    info!("Loaded Release Manifest for: \"{}\"", version_entry.id);

    // Get the `AssetManifest`
    let asset_manifest =
        match froglight_tools::manifests::get_asset_manifest(&release_manifest, &cache, &client)
            .await
        {
            Ok(manifest) => manifest,
            Err(err) => {
                error!("Failed to get `AssetManifest`: {err}");
                return Err(err.into());
            }
        };
    info!("Loaded Asset Manifest for: \"{}\"", version_entry.id);
    debug!("Asset Manifest: {} Assets", asset_manifest.objects.len());

    // Download the `Client` JAR
    let Some(client_jar) = froglight_tools::deobfuscate::download_client_jar(
        &release_manifest.downloads,
        &cache,
        &client,
    )
    .await
    else {
        error!("Failed to download `Client` JAR");
        return Err(anyhow!("Failed to download `Client` JAR"));
    };

    // Download the `Server` JAR
    let Some(server_jar) =
        froglight_tools::json::download_server_jar(&release_manifest.downloads, &cache, &client)
            .await
    else {
        error!("Failed to download `Server` JAR");
        return Err(anyhow!("Failed to download `Server` JAR"));
    };

    // --- Deobfuscate Jar ---

    // Get the latest Yarn mappings for this version
    let Some(yarn_build) = yarn_manifest.get_latest(&version_entry.id) else {
        error!("No Yarn mappings found for: \"{}\"", version_entry.id);
        return Err(anyhow!("No Yarn mappings found"));
    };
    info!("Using Yarn: \"{yarn_build}\"");

    let Some(yarn_mappings) =
        froglight_tools::mappings::get_yarn_mappings(yarn_build, &cache, &client).await
    else {
        error!("Failed to download `Yarn` mappings");
        return Err(anyhow!("Failed to download `Yarn` mappings"));
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
        error!("Failed to deobfuscate `Client` JAR");
        return Err(anyhow!("Failed to deobfuscate `Client` JAR"));
    };

    // Read and parse the deobfuscated `Client` JAR
    info!("Parsing \"{}\" `Client` JAR ...", version_entry.id);
    let jar_reader = match ZipFileReader::new(mapped_jar).await {
        Ok(reader) => reader,
        Err(err) => {
            error!("Failed to create ZIP reader: {err}");
            return Err(err.into());
        }
    };
    let jar_container = match JarContainer::new_tokio_fs(&jar_reader).await {
        Ok(jar) => jar,
        Err(err) => {
            error!("Failed to parse `Client` JAR: {err}");
            return Err(err);
        }
    };
    info!("Successfully parsed \"{}\" `Client` JAR", version_entry.id);
    debug!("\"{}\" parsed {} classes", version_entry.id, jar_container.len());

    // --- Extract  ---

    // Run the `Server` JAR generators
    info!("Running \"{}\" `Server` JAR generators ...", version_entry.id);
    let Some(json_path) = froglight_tools::json::generate_server_json(&server_jar, &cache).await
    else {
        error!("Failed to generate `Server` JSON");
        return Err(anyhow!("Failed to generate `Server` JSON"));
    };
    info!("Generated \"{}\" `Server` JSON files", version_entry.id);

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

    // Sort modules and extract data
    let mut modules = args.modules.clone();
    if modules.is_empty() {
        modules.extend(Modules::DEFAULT);
    }
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
    Ok(extract_data)
}
