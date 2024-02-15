use std::path::Path;

use async_zip::{
    tokio::{read::fs::ZipFileReader, write::ZipFileWriter},
    Compression, ZipEntryBuilder,
};
use froglight_data::{AssetManifest, Version};
use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{fs::File, io::AsyncWrite};
use tracing::{debug, error, info, trace};

use super::Extract;
use crate::{
    classmap::ClassMap,
    manifest::{asset_manifest, release_manifest, version_manifest},
};

/// A module that extracts assets and asset metadata.
///
/// This includes things like textures, models, and sounds.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct AssetModule;

impl Extract for AssetModule {
    async fn extract(
        &self,
        version: &Version,
        _: &ClassMap,
        cache: &Path,
        _: &mut Value,
    ) -> anyhow::Result<()> {
        // Get the release manifest and release information
        let version_manifest = version_manifest(cache, false).await?;
        let version_information =
            version_manifest.get(version).expect("Version not in VersionManifest");

        // Get the release manifest
        let release_manifest = release_manifest(version_information, cache, false).await?;
        let asset_manifest = asset_manifest(version, &release_manifest, cache, false).await?;

        // Update the cache path
        let mut cache = cache.join("froglight");
        cache.push(version.to_short_string());

        // Create a new zip file for the assets
        let zip_path = cache.join("assets.zip");
        info!("Assets Path: {}", zip_path.display());

        let file_writer = File::create(&zip_path).await?;
        let mut zip_writer = ZipFileWriter::with_tokio(file_writer);

        // Copy assets from the minecraft jar
        copy_assets(&cache, &asset_manifest, &mut zip_writer).await?;

        // Download assets from the asset manifest
        download_assets(&asset_manifest, &mut zip_writer).await?;

        // Finish writing the zip
        zip_writer.close().await?;

        Ok(())
    }
}

/// Open the minecraft jar and copy all assets into the zip.
async fn copy_assets(
    cache: &Path,
    asset_manifest: &AssetManifest,
    zip_writer: &mut ZipFileWriter<File>,
) -> anyhow::Result<()> {
    // Open `client.jar`
    let jar_path = cache.join("client.jar");
    let jar = ZipFileReader::new(jar_path).await?;

    // Copy all assets from the jar to the zip
    let entries = jar.file().entries().len();
    for index in 0..entries {
        let Ok(mut reader) = jar.reader_with_entry(index).await else {
            error!("Failed to read entry `{index}` from `client.jar`");
            continue;
        };

        // Get the filename of the entry
        let filename = reader.entry().filename().as_str().unwrap().to_string();

        // Skip files that can be downloaded from the asset manifest
        for key in asset_manifest.objects.keys() {
            if key.contains(&filename) {
                info!("Skipping `{filename}` as it can be downloaded");

                continue;
            }
        }

        // Only copy files from the `assets` directory or files that contain `pack.`
        if !filename.starts_with("assets/") && !filename.contains("pack.") {
            info!("Skipping `{filename}` as it's not in the `assets` directory or a `pack.*` file");

            continue;
        }

        // Only copy files that don't end with `.class`
        if Path::new(&filename).extension().map_or(false, |ext| ext.eq_ignore_ascii_case("class")) {
            info!("Skipping `{filename}` as it's a class file");

            continue;
        }

        trace!("Copying `{filename}`");

        // Log every 10 entries
        if index % 100 == 0 {
            info!("Copying... {index}/{entries}");
        }

        // Read the entry into memory
        let mut data = Vec::new();
        reader.read_to_end_checked(&mut data).await?;

        // Copy the entry to the zip
        let entry = ZipEntryBuilder::new(filename.clone().into(), Compression::Deflate).build();
        if let Err(err) = zip_writer.write_entry_whole(entry, data.as_slice()).await {
            error!("Failed to write `{filename}` to `assets.zip`: `{err}`");
        }
    }

    Ok(())
}

/// Download all assets from the asset manifest and add them to the zip.
async fn download_assets(
    asset_manifest: &AssetManifest,
    zip_writer: &mut ZipFileWriter<File>,
) -> anyhow::Result<()> {
    // Create a client for all reqwest https requests
    let client = ClientBuilder::new().build()?;

    // Download and add all assets to the zip
    let count = asset_manifest.objects.len();
    for (index, (name, asset)) in asset_manifest.objects.iter().enumerate() {
        // Log progress every 10 assets
        if index % 100 == 0 {
            info!("Downloading... {index}/{count}");
        }

        if let Err(err) = add_asset_to_zip(name, &asset.url(), zip_writer, &client).await {
            error!("Failed to download `{name}`: `{err}`");
        } else {
            debug!("Downloaded `{name}`");
        }
    }

    Ok(())
}

/// Download an asset and add it to the zip.
async fn add_asset_to_zip<W>(
    name: &str,
    asset_url: &str,
    zip: &mut ZipFileWriter<W>,
    client: &Client,
) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    // Download the asset
    let response = client.get(asset_url).send().await?;
    let bytes = response.bytes().await?;

    let entry = if name.contains("pack.") {
        // Add the asset to the root of the zip
        ZipEntryBuilder::new(name.into(), Compression::Deflate).build()
    } else {
        // Add the asset to the zip
        ZipEntryBuilder::new(format!("assets/{name}").into(), Compression::Deflate).build()
    };
    zip.write_entry_whole(entry, &bytes).await?;

    Ok(())
}
