use std::sync::LazyLock;

use compact_str::CompactString;
use hashbrown::HashMap;
use reqwest::Client;

use super::{BlockSpecification, BlockSpecificationState};
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

        // Check all blocks have valid data.
        blocks.iter().for_each(|block| {
            assert!(!block.name.is_empty(), "v{version} has a block no name!");
            assert!(
                !block.display_name.is_empty(),
                "v{version} {} has no display name!",
                block.name
            );
            assert!(!block.material.is_empty(), "v{version} {} has no material!", block.name);
            assert!(
                !block.bounding_box.is_empty(),
                "v{version} {} has no bounding box!",
                block.name
            );

            block.states.iter().for_each(|state| match state {
                BlockSpecificationState::Bool { num_values, .. } => assert_eq!(
                    *num_values, 2,
                    "v{version} {} has a bool state with {num_values} values!",
                    block.name
                ),
                BlockSpecificationState::Enum { num_values, values, .. } => {
                    assert!(
                        !values.is_empty(),
                        "v{version} {} has an enum state with no values!",
                        block.name
                    );
                    assert_eq!(
                        *num_values as usize,
                        values.len(),
                        "v{version} {} has an enum state with a different number of values!",
                        block.name
                    );
                    for value in values {
                        assert!(
                            !value.is_empty(),
                            "v{version} {} has an enum state with an empty value!",
                            block.name
                        );
                    }
                }
                BlockSpecificationState::Int { num_values, values, .. } => {
                    assert!(
                        !values.is_empty(),
                        "v{version} {} has an int state with no values!",
                        block.name
                    );
                    assert_eq!(
                        *num_values as usize,
                        values.len(),
                        "v{version} {} has an int state with a different number of values!",
                        block.name
                    );
                }
            });
        });
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
    assert_eq!(b1_21_0, b1_21_1, "v1.21.0 and v1.21.1 have different blocks!");

    // Make sure the blocks are correct.
    assert_eq!(b1_20_6.get(CHERRY_PRESSURE_PLATE.id as usize), Some(&*CHERRY_PRESSURE_PLATE));
    assert_eq!(b1_20_6.get(BRAIN_CORAL.id as usize), Some(&*BRAIN_CORAL));

    assert_eq!(b1_21_0.get(CHERRY_PRESSURE_PLATE.id as usize), Some(&*CHERRY_PRESSURE_PLATE));
    assert_eq!(b1_21_0.get(BRAIN_CORAL.id as usize), Some(&*BRAIN_CORAL));

    assert_eq!(b1_21_1.get(CHERRY_PRESSURE_PLATE.id as usize), Some(&*CHERRY_PRESSURE_PLATE));
    assert_eq!(b1_21_1.get(BRAIN_CORAL.id as usize), Some(&*BRAIN_CORAL));
}

static CHERRY_PRESSURE_PLATE: LazyLock<BlockSpecification> = LazyLock::new(|| BlockSpecification {
    id: 238,
    name: CompactString::const_new("cherry_pressure_plate"),
    display_name: CompactString::const_new("Cherry Pressure Plate"),
    hardness: 0.5,
    resistance: 0.5,
    stack_size: 64,
    diggable: true,
    material: CompactString::const_new("mineable/axe"),
    transparent: true,
    emit_light: 0,
    filter_light: 0,
    default_state: 5727,
    min_state_id: 5726,
    max_state_id: 5727,
    states: vec![BlockSpecificationState::Bool {
        name: CompactString::const_new("powered"),
        num_values: 2,
    }],
    harvest_tools: HashMap::new(),
    drops: vec![704],
    bounding_box: CompactString::const_new("empty"),
});

static BRAIN_CORAL: LazyLock<BlockSpecification> = LazyLock::new(|| BlockSpecification {
    id: 699,
    name: CompactString::const_new("brain_coral"),
    display_name: CompactString::const_new("Brain Coral"),
    hardness: 0.0,
    resistance: 0.0,
    stack_size: 64,
    diggable: true,
    material: CompactString::const_new("default"),
    transparent: true,
    emit_light: 0,
    filter_light: 1,
    default_state: 12825,
    min_state_id: 12825,
    max_state_id: 12826,
    states: vec![BlockSpecificationState::Bool {
        name: CompactString::const_new("waterlogged"),
        num_values: 2,
    }],
    harvest_tools: HashMap::new(),
    drops: Vec::new(),
    bounding_box: CompactString::const_new("empty"),
});
