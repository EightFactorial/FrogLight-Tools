use reqwest::Client;

use crate::{
    file::{FileTrait, GeneratorData, VersionInfo, VersionManifest},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_21_0 = Version::new_release(1, 21, 0);
    let v1_21_1 = Version::new_release(1, 21, 1);

    let manifest = VersionManifest::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    // Fetch the data for v1.21.0
    let i1_21_0 = VersionInfo::fetch(&v1_21_0, &cache, &manifest, false, &client).await.unwrap();
    let g1_21_0 = GeneratorData::fetch(&v1_21_0, &cache, &i1_21_0, false, &client).await.unwrap();

    // Fetch the data for v1.21.1
    let i1_21_1 = VersionInfo::fetch(&v1_21_1, &cache, &manifest, false, &client).await.unwrap();
    let g1_21_1 = GeneratorData::fetch(&v1_21_1, &cache, &i1_21_1, false, &client).await.unwrap();

    // Assert that the assets, reports, and data are the same for both versions.
    // Note: This may fail when more data is parsed
    assert_eq!(g1_21_0.assets, g1_21_1.assets);
    assert_eq!(g1_21_0.reports, g1_21_1.reports);
    assert_eq!(g1_21_0.data, g1_21_1.data);
}
