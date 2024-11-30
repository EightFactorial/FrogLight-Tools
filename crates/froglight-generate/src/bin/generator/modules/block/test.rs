use std::hash::{DefaultHasher, Hash, Hasher};

use convert_case::{Case, Casing};
use froglight_generate::{CliArgs, DataMap};
use froglight_parse::{file::VersionBlocks, Version};
use itertools::Itertools;

const NUMBER_OF_TESTS: usize = 25;

pub(super) async fn generate_tests(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
    for (version, data) in &datamap.version_data {
        generate_version_test(version, &data.blocks, args).await?;
    }

    let module_names: String = datamap
        .version_data
        .keys()
        .map(|v| format!("mod v{};", v.to_long_string().replace(['.'], "_")))
        .join("\n");

    let file_path = args.dir.join("crates/froglight-block/src/test/mod.rs");
    if !file_path.exists() {
        tracing::warn!("BlockGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }
    tokio::fs::write(file_path, module_names).await?;

    Ok(())
}

async fn generate_version_test(
    version: &Version,
    blocks: &VersionBlocks,
    args: &CliArgs,
) -> anyhow::Result<()> {
    let version_name = format!("V{}", version.to_long_string().replace(['.'], "_"));
    let module_name = version_name.to_ascii_lowercase();

    let mut content = format!(
        r"//! Generated tests for [`{version_name}`].
//!
//! @generated by 'TODO'
#![allow(clippy::wildcard_imports)]
use std::any::TypeId;

use bevy::MinimalPlugins;
use bevy_app::App;
use froglight_protocol::versions::{module_name}::{version_name};

use crate::{{block::*, BlockPlugin, BlockStorageArc}};

#[test]
#[expect(clippy::too_many_lines)]
fn generated() {{
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(BlockPlugin);
    app.finish();

    // Retrieve the block storage.
    let storage = app.world().resource::<BlockStorageArc<{version_name}>>();
    let storage = storage.read();

"
    );

    // Hash the version number, use it as the seed.
    let mut hasher = DefaultHasher::new();
    version.hash(&mut hasher);
    // Generate random block IDs.
    let mut block_indexes = Vec::with_capacity(NUMBER_OF_TESTS);
    for i in 0..NUMBER_OF_TESTS {
        hasher.write_usize(i);
        block_indexes.push(hasher.finish() % blocks.len() as u64);
    }

    // Generate the test cases.
    for index in block_indexes {
        let block = &blocks[usize::try_from(index).unwrap()];
        let block_name = block.name.replace(['\''], "_").to_case(Case::Pascal);
        let block_id = block.min_state_id;
        let default_id = block.default_state;

        content.push_str(&format!(
            r#"    if let Some(block) = storage.get_stored_default({block_id}u32) {{
        assert_eq!(block.resource_key(), "minecraft:{}");
        assert_eq!(block.type_id(), TypeId::of::<{block_name}>());

        let downcast = block.as_any().downcast_ref::<{block_name}>().unwrap();
        assert_eq!(storage.get_block_id(downcast), Some({default_id}u32));
    }}
"#,
            block.name
        ));
    }

    content.push_str("\n}");

    // Write the output to a file.
    let file_path =
        args.dir.join("crates/froglight-block/src/test").join(format!("{module_name}.rs",));
    if !file_path.exists() {
        tracing::warn!("BlockGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }
    tokio::fs::write(file_path, &content).await?;

    Ok(())
}
