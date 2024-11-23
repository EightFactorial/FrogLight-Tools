use convert_case::{Case, Casing};
use froglight_generate::{CliArgs, DataMap};
use froglight_parse::{file::VersionBlocks, Version};
use itertools::Itertools;

pub(super) async fn generate_storage(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
    for (version, data) in &datamap.version_data {
        generate_storage_version(version, &data.blocks, args).await?;
    }

    let mut version_names: Vec<_> = datamap.version_data.keys().map(|v|
        format!("V{}", v.to_long_string().replace(['.'], "_"))
    ).collect();
    version_names.sort_unstable();
    let version_modules: Vec<_> = version_names.iter().map(|v| v.to_ascii_lowercase()).collect();


    let imports = version_modules.iter().zip_eq(version_names.iter()).map(|(module, version)| {
        format!("{module}::{version}")
    }).join(", ");

    let modules = version_modules.iter().map(|module| {
        format!("mod {module};")
    }).join("\n");

    let registrations = version_names.iter().map(|version| {
        format!("        app.register_type_data::<Self, ReflectBlockBuilder<{version}>>();\n        app.init_resource::<BlockStorageArc<{version}>>();")
    }).join("\n");


    let content = format!(
r"use bevy_app::App;
use bevy_reflect::Reflect;
use froglight_protocol::versions::{{{imports}}};

use super::{{BlockStorageArc, ReflectBlockBuilder}};

{modules}

/// A builder for vanilla block storage.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct VanillaBuilder;

impl VanillaBuilder {{
    pub(super) fn build(app: &mut App) {{
        app.register_type::<Self>();
{registrations}
    }}
}}
");

    let file_path = args.dir.join("crates/froglight-block/src/storage/vanilla/mod.rs");
    if !file_path.exists() {
        tracing::warn!("BlockGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }
    tokio::fs::write(file_path, &content).await?;

    Ok(())
}

async fn generate_storage_version(version: &Version, blocks: &VersionBlocks, args: &CliArgs) -> anyhow::Result<()> {
    let version_name = format!("V{}", version.to_long_string().replace(['.'], "_"));
    let module_name = version_name.to_ascii_lowercase();

    let mut registrations = String::new();
    for (index, block) in blocks.iter().enumerate() {
        let block_name = block.name.replace(['\''], "_").to_case(Case::Pascal);
        registrations.push_str(&format!("        storage.register::<{block_name}>();"));

        if index != blocks.len() - 1 {
            registrations.push('\n');
        }
    }

    let content = format!(
r"//! [`VanillaBuilder`] for [`{version_name}`].
//! 
//! @generated by 'TODO'
#![allow(clippy::wildcard_imports)]

use bevy_ecs::world::World;
use froglight_protocol::versions::{module_name}::{version_name};

use super::VanillaBuilder;
use crate::{{block::*, BlockBuilder, BlockStorage, ReflectBlockBuilder}};

impl BlockBuilder<{version_name}> for VanillaBuilder {{
    #[expect(clippy::too_many_lines)]
    fn build(
        storage: &mut BlockStorage<{version_name}>,
        _: &mut World,
        _: &[&ReflectBlockBuilder<{version_name}>],
    ) {{
{registrations}
    }}
}}
");

    let file_path = args.dir.join(format!("crates/froglight-block/src/storage/vanilla/{module_name}.rs"));
    if !file_path.exists() {
        tracing::warn!("BlockGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }
    tokio::fs::write(file_path, &content).await?;

    Ok(())
}
