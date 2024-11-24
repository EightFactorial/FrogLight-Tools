use reqwest::Client;

use super::VersionEntities;
use crate::{
    file::{DataPath, FileTrait},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_21_1 = Version::new_release(1, 21, 1);
    let datapaths = DataPath::fetch(&v1_21_1, &cache, &(), false, &client).await.unwrap();

    let e1_21_1 =
        VersionEntities::fetch(&v1_21_1, &cache, &datapaths, false, &client).await.unwrap();

    assert_eq!(e1_21_1.len(), 130);
    for (index, entity) in e1_21_1.iter().enumerate() {
        assert_eq!(entity.id, u32::try_from(index).unwrap());
    }

    let allay = e1_21_1.first().unwrap();
    assert_eq!(allay.name, "allay");
    assert_eq!(allay.display_name, "Allay");
    assert_eq!(allay.width.total_cmp(&0.35), std::cmp::Ordering::Equal);
    assert_eq!(allay.height.total_cmp(&0.6), std::cmp::Ordering::Equal);
    assert_eq!(allay.kind, "mob");
    assert_eq!(allay.category, "Passive mobs");
}
