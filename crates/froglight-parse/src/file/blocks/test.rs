use reqwest::Client;

use crate::{
    file::{DataPath, FileTrait, VersionBlocks},
    Version,
};

#[tokio::test]
async fn fetch() {
    let client = Client::new();
    let cache = crate::file::target_dir().await;

    let v1_20_6 = Version::new_release(1, 20, 6);
    let v1_21_0 = Version::new_release(1, 21, 0);
    let v1_21_1 = Version::new_release(1, 21, 1);
    let datapaths = DataPath::fetch(&v1_21_0, &cache, &(), false, &client).await.unwrap();

    // Fetch the blocks for multiple versions.
    let b1_20_6 = VersionBlocks::fetch(&v1_20_6, &cache, &datapaths, false, &client).await.unwrap();
    let b1_21_0 = VersionBlocks::fetch(&v1_21_0, &cache, &datapaths, false, &client).await.unwrap();
    let b1_21_1 = VersionBlocks::fetch(&v1_21_1, &cache, &datapaths, false, &client).await.unwrap();

    // Make sure the blocks are not empty.
    for (version, blocks) in [(&v1_20_6, &b1_20_6), (&v1_21_0, &b1_21_0), (&v1_21_1, &b1_21_1)] {
        assert!(!blocks.is_empty(), "v{version} has no blocks!");
    }

    // Make sure the blocks are *mostly* the same between v1.20.6 and v1.21.0.
    for (block_a, block_b) in b1_20_6.iter().filter_map(|block| {
        b1_21_0.iter().find(|check| block.name == check.name).map(|check| (block, check))
    }) {
        assert_eq!(block_a.bounding_box, block_b.bounding_box);
        assert_eq!(block_a.diggable, block_b.diggable);
        assert_eq!(block_a.id, block_b.id);
        assert_eq!(block_a.name, block_b.name);
        assert_eq!(block_a.stack_size, block_b.stack_size);
        assert_eq!(block_a.states, block_b.states);
    }

    // Make sure blocks between v1.21.0 and v1.21.1 are completely identical.
    assert_eq!(b1_21_0, b1_21_1, "v1.21.0 and v1.21.1 have different block counts");
}
