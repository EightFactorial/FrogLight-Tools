use std::path::{Path, PathBuf};

use reqwest::Client;

use crate::{file::FileTrait, Version};

impl super::VersionManifest {
    /// The name of the manifest file.
    pub const FILE_NAME: &str = "version_manifest_v2.json";
    /// The URL of the manifest file.
    pub const FILE_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
}

impl FileTrait for super::VersionManifest {
    type UrlData = ();
    fn get_url(_: &Version, (): &Self::UrlData) -> String { Self::FILE_URL.to_string() }
    fn get_path(_: &Version, cache: &Path) -> PathBuf { cache.join(Self::FILE_NAME) }

    /// Fetch the manifest file, downloading it if it doesn't exist.
    ///
    /// # Note
    /// Version is ignored because the manifest file is the same for all
    /// versions.
    fn fetch(
        version: &Version,
        cache: &Path,
        data: &Self::UrlData,
        redownload: bool,
        client: &Client,
    ) -> impl std::future::Future<Output = anyhow::Result<Self>> {
        crate::file::fetch_json(version, cache, data, redownload, client)
    }
}
