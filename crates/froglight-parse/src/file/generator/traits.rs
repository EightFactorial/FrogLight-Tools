use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

use reqwest::Client;
use tokio::process::Command;

use super::{GeneratedAssets, GeneratedData, GeneratedReports};
use crate::{
    file::{FileTrait, VersionInfo},
    Version,
};

impl FileTrait for super::GeneratorData {
    type UrlData = VersionInfo;
    fn get_url(_: &Version, data: &Self::UrlData) -> Option<String> {
        Some(data.downloads.server.url.to_string())
    }
    fn get_path(version: &Version, cache: &Path) -> PathBuf {
        cache.join(format!("v{version}")).join("server.jar")
    }

    async fn fetch(
        version: &Version,
        cache: &Path,
        data: &Self::UrlData,
        redownload: bool,
        client: &Client,
    ) -> anyhow::Result<Self> {
        // Emit a warning if the version is below `1.21.0`
        match version.compare_relative(&Version::new_release(1, 21, 0)) {
            Some(Ordering::Less) => {
                tracing::warn!("Version v{version} is before v1.21.0, this may not work!");
            }
            None => {
                tracing::warn!(
                    "Version \"{version}\" is not a release version, this may not work!"
                );
            }
            _ => {}
        }

        // Fetch the server jar.
        let path =
            crate::file::fetch_file::<Self>(version, cache, data, redownload, client).await?;

        let generator_cache = path.parent().unwrap().join("generator-cache");
        let generator = path.parent().unwrap().join("generator");

        // Remove the existing cached data if redownload is set.
        if redownload {
            tokio::fs::remove_dir_all(&generator_cache).await?;
            tokio::fs::remove_dir_all(&generator).await?;
        }

        // Run the data generator if the data doesn't exist.
        if !generator.exists() {
            tokio::fs::create_dir_all(&generator_cache).await?;
            tokio::fs::create_dir_all(&generator).await?;

            let mut child = Command::new("java")
                .current_dir(generator_cache)
                .stdout(std::io::stderr())
                .arg("-DbundlerMainClass=net.minecraft.data.Main")
                .arg("-jar")
                // Relative to `generator_cache`
                .arg("../server.jar")
                .arg("--all")
                .arg("--output")
                // Relative to `generator_cache`
                .arg("../generator")
                .spawn()?;

            // Wait for the generator to finish, bail if it fails.
            if !child.wait().await?.success() {
                anyhow::bail!("Failed to generate data");
            }
        }

        // Read the generated assets.
        tracing::trace!("Parsing Assets: {version}");
        let assets = GeneratedAssets::new(&generator.join("assets")).await?;

        tracing::trace!("Parsing Data: {version}");
        let data = GeneratedData::new(&generator.join("data")).await?;

        tracing::trace!("Parsing Reports: {version}");
        let reports = GeneratedReports::new(&generator.join("reports")).await?;

        tracing::trace!("Finished Parsing: {version}");
        Ok(Self { assets, data, reports })
    }
}
