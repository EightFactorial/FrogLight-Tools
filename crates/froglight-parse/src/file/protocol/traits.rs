use std::path::{Path, PathBuf};

use reqwest::Client;

use crate::{
    file::{DataPath, FileTrait},
    Version,
};

impl super::VersionProtocol {
    /// The name of the protocol file.
    pub const FILE_NAME: &str = "protocol.json";
}

impl FileTrait for super::VersionProtocol {
    type UrlData = DataPath;
    fn get_url(version: &Version, data: &Self::UrlData) -> String {
        data.get_java_protocol(version).expect("Version not found")
    }

    fn get_path(version: &Version, cache: &Path) -> PathBuf {
        cache.join(format!("v{version}")).join(Self::FILE_NAME)
    }

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
