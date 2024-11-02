use reqwest::Client;

use crate::{
    file::{FileTrait, MavenMetadata},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_21_0 = Version::new_release(1, 21, 0);

    // Fetch the MavenMetadata
    let manifest = MavenMetadata::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    assert_eq!(manifest.group_id, "net.fabricmc");
    assert_eq!(manifest.artifact_id, "yarn");
    assert!(!manifest.versioning.versions.is_empty());
    assert_ne!(manifest.versioning.last_updated, 0);
}
