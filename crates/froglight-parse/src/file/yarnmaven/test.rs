use reqwest::Client;

use crate::{
    file::{FileTrait, YarnMavenMetadata},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_21_0 = Version::new_release(1, 21, 0);

    // Fetch the YarnMavenMetadata
    let metadata = YarnMavenMetadata::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    assert_eq!(metadata.group_id, "net.fabricmc");
    assert_eq!(metadata.artifact_id, "yarn");
    assert!(!metadata.versioning.versions.is_empty());
    assert_ne!(metadata.versioning.last_updated, 0);
}
