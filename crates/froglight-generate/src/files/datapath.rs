use std::path::{Path, PathBuf};

use froglight_parse::{files::DataPaths, Version};
use reqwest::Client;

use super::FileTrait;

impl FileTrait for DataPaths {
    fn get_url() -> &'static str { Self::FILE_URL }
    fn get_path(_: &Version, cache: &Path) -> PathBuf { cache.join(Self::FILE_NAME) }

    fn fetch(
        version: &Version,
        cache: &Path,
        redownload: bool,
        client: &Client,
    ) -> impl std::future::Future<Output = anyhow::Result<Self>> + Send + Sync {
        super::fetch_json(version, cache, redownload, client)
    }
}

#[tokio::test]
async fn fetch() {
    // Find the target directory.
    let mut cache = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    while !cache.join("target").exists() {
        assert!(!cache.to_string_lossy().is_empty(), "Couldn't find target directory");
        cache = cache.parent().unwrap().to_path_buf();
    }

    cache.push("target");
    cache.push("froglight-generate");
    tokio::fs::create_dir_all(&cache).await.unwrap();

    // Fetch the DataPaths
    let version = Version::new_release(1, 21, 1);
    let client = Client::new();

    let datapaths = DataPaths::fetch(&version, &cache, false, &client).await.unwrap();

    assert_eq!(
        datapaths.get_java_proto(&Version::new_release(1, 20, 0)).as_deref(),
        Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.20/proto.yml")
    );
    assert_eq!(datapaths.get_java_proto(&Version::new_release(1, 20, 1)).as_deref(), None);
    assert_eq!(
        datapaths.get_java_proto(&Version::new_release(1, 20, 2)).as_deref(),
        Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.20.2/proto.yml")
    );
    assert_eq!(
        datapaths.get_java_proto(&Version::new_release(1, 20, 3)).as_deref(),
        Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.20.3/proto.yml")
    );
    assert_eq!(
        datapaths.get_java_proto(&Version::new_release(1, 20, 4)).as_deref(),
        Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.20.3/proto.yml")
    );
    assert_eq!(
        datapaths.get_java_proto(&Version::new_release(1, 20, 5)).as_deref(),
        Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.20.5/proto.yml")
    );
    assert_eq!(
        datapaths.get_java_proto(&Version::new_release(1, 20, 6)).as_deref(),
        Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.20.5/proto.yml")
    );
    // assert_eq!(
    //     datapaths.get_java_proto(&Version::new_release(1, 21, 0)).as_deref(),
    //     Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.21/proto.yml")
    // );
    // assert_eq!(
    //     datapaths.get_java_proto(&Version::new_release(1, 21, 1)).as_deref(),
    //     Some("https://raw.githubusercontent.com/PrismarineJS/minecraft-data/refs/heads/master/data/pc/1.21/proto.yml")
    // );
}
