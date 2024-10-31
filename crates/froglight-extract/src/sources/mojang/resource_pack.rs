use async_zip::{tokio::write::ZipFileWriter, Compression, ZipEntryBuilder};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tracing::{error, info, trace, warn};

use crate::{bundle::ExtractBundle, sources::ExtractModule};

/// A module that adds debug information to the output.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize_unit_struct,
    Serialize_unit_struct,
)]
pub struct ResourcePack;

impl ExtractModule for ResourcePack {
    async fn extract(&self, data: &mut ExtractBundle) -> anyhow::Result<()> {
        let pack_path = data.cache_dir.join("resourcepack.zip");
        info!("Creating ResourcePack at: \"{}\"", pack_path.display());

        if pack_path.exists() {
            warn!("Overwriting existing ResourcePack...");
            tokio::fs::remove_file(&pack_path).await?;
        }

        // Create the resource pack
        let file_writer = tokio::fs::File::create(&pack_path).await?;
        let mut zip_writer = ZipFileWriter::with_tokio(file_writer);

        // Copy the assets folder from the client jar
        info!("Copying assets from client jar...");
        for index in 0..data.jar_reader.file().entries().len() {
            let mut entry = data.jar_reader.reader_with_entry(index).await?;
            let path = entry.entry().filename().as_str()?;

            if path.starts_with("assets/") {
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer).await?;

                zip_writer.write_entry_whole(entry.entry().clone(), &buffer).await?;
            }
        }

        let mut next_log = data.manifests.asset.objects.len() / 10;
        let total_objects = data.manifests.asset.objects.len();

        // Download assets from the server
        let client = reqwest::Client::new();
        info!("Downloading assets from server: 0/{total_objects}");
        for (index, (path, object)) in data.manifests.asset.objects.iter().enumerate() {
            let mut path = path.to_string();
            if path != "pack.mcmeta" {
                path = format!("assets/{path}");
            }

            let url = object.get_url();
            let response = client.get(&url).send().await?;

            if response.status().is_success() {
                let buffer = response.bytes().await?;
                trace!("Downloaded: \"{path}\"");

                if buffer.is_empty() {
                    error!("Received empty response for \"{path}\"");
                    continue;
                }

                let entry = ZipEntryBuilder::new(path.into(), Compression::Deflate).build();
                let mut entry_writer = zip_writer.write_entry_stream(entry).await?;

                entry_writer.write_all(&buffer).await?;
                entry_writer.close().await?;
            } else {
                error!("Failed to download \"{path}\": {}", response.status());
            }

            // Log progress every 10%
            if index >= next_log {
                info!("Downloading assets from server: {index}/{total_objects}");
                next_log += total_objects / 10;
            }
        }
        zip_writer.close().await?;

        info!("ResourcePack written to: \"{}\"", pack_path.display());

        Ok(())
    }
}
