use std::path::{Path, PathBuf};

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
        Ok(Self {
            assets: GeneratedAssets::new(&generator.join("assets")).await?,
            data: GeneratedData::new(&generator.join("data")).await?,
            reports: GeneratedReports::new(&generator.join("reports")).await?,
        })
    }
}
