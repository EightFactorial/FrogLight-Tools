use std::path::{Path, PathBuf};

use reqwest::Client;

use crate::{file::FileTrait, Version};

impl FileTrait for super::DataPath {
    type UrlData = ();
    fn get_url(_: &Version, (): &Self::UrlData) -> String { Self::FILE_URL.to_string() }
    fn get_path(_: &Version, cache: &Path) -> PathBuf { cache.join(Self::FILE_NAME) }

    /// Fetch the data paths file, downloading it if it doesn't exist.
    ///
    /// # Note
    /// Version is ignored because the data paths file is the same for all
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
