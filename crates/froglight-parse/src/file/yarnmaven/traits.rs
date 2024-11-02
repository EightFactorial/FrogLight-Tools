use std::path::{Path, PathBuf};

use reqwest::Client;

use crate::{file::FileTrait, Version};

impl super::YarnMavenMetadata {
    /// The name of the Maven metadata file.
    pub const FILE_NAME: &str = "maven-metadata.json";
    /// The URL of the Maven metadata file.
    pub const FILE_URL: &str = "https://maven.fabricmc.net/net/fabricmc/yarn/maven-metadata.xml";
}

impl FileTrait for super::YarnMavenMetadata {
    type UrlData = ();
    fn get_url(_: &Version, (): &Self::UrlData) -> String { Self::FILE_URL.to_string() }
    fn get_path(_: &Version, cache: &Path) -> PathBuf { cache.join(Self::FILE_NAME) }

    fn fetch(
        version: &Version,
        cache: &Path,
        data: &Self::UrlData,
        redownload: bool,
        client: &Client,
    ) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send + Sync {
        crate::file::fetch_xml(version, cache, data, redownload, client)
    }
}
