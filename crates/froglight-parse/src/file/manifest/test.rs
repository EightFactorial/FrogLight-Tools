use reqwest::Client;

use crate::{
    file::{manifest::ReleaseType, FileTrait, VersionManifest},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    // Fetch the VersionManifest
    let v1_21_0 = Version::new_release(1, 21, 0);
    let manifest = VersionManifest::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    // Check the release types of all versions in the manifest
    for info in manifest.versions.values() {
        // Known mismatches, these versions are labeled as snapshots
        if matches!(
            info.id.to_long_string().as_str(),
            "1.3.0" | "1.4.0" | "1.4.1" | "1.4.3" | "1.5.0" | "1.6.0" | "1.6.3" | "1.7.0" | "1.7.1"
        ) {
            assert_eq!(info.kind, ReleaseType::Snapshot);
            continue;
        }

        match (&info.id, &info.kind) {
            // Release builds must be marked "release"
            (Version::Release(..), ReleaseType::Release)
            // Non-release builds must be marked "snapshot"
            | (Version::ReleaseCandidate(..) | Version::PreRelease(..) | Version::Snapshot(..), ReleaseType::Snapshot)
            // Ignore unknown version types
            | (Version::Other(..), ..) => {}
            (version, kind) => panic!("Status does not match, {version} is marked {kind:?}?"),
        }
    }
}
