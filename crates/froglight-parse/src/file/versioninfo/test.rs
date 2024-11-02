use reqwest::Client;

use crate::{
    file::{manifest::ReleaseType, FileTrait, VersionInfo, VersionManifest},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_21_0 = Version::new_release(1, 21, 0);
    let v1_21_1 = Version::new_release(1, 21, 1);

    // Fetch the VersionManifest
    let manifest = VersionManifest::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    // Fetch the VersionInfo
    let i1_21_0 = VersionInfo::fetch(&v1_21_0, &cache, &manifest, false, &client).await.unwrap();
    let i1_21_1 = VersionInfo::fetch(&v1_21_1, &cache, &manifest, false, &client).await.unwrap();

    assert_eq!(i1_21_0.id, v1_21_0);
    assert_eq!(i1_21_0.kind, ReleaseType::Release);
    assert!(!i1_21_0.downloads.client.url.is_empty());

    assert_eq!(i1_21_1.id, v1_21_1);
    assert_eq!(i1_21_1.kind, ReleaseType::Release);
    assert!(!i1_21_1.downloads.client.url.is_empty());
}
