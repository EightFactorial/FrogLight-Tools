use std::path::{Path, PathBuf};

use reqwest::Client;

use crate::{
    file::{FileTrait, VersionManifest},
    Version,
};

impl FileTrait for super::VersionInfo {
    type UrlData = VersionManifest;
    fn get_url(version: &Version, data: &Self::UrlData) -> String {
        data.versions.get(version).unwrap().url.clone().into()
    }

    fn get_path(version: &Version, cache: &Path) -> PathBuf {
        cache.join(format!("v{version}")).join(format!("{version}.json"))
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
