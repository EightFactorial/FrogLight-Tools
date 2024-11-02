use reqwest::Client;

use crate::{
    file::{DataPath, FileTrait},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    // Fetch the DataPaths
    let version = Version::new_release(1, 21, 1);
    let datapaths = DataPath::fetch(&version, &cache, &(), false, &client).await.unwrap();

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
