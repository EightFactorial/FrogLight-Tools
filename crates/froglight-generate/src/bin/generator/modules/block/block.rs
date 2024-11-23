use convert_case::{Case, Casing};
use froglight_generate::{BlockGenerator, CliArgs, DataMap};
use froglight_parse::{file::VersionBlocks, Version};
use hashbrown::HashMap;
use proc_macro2::Span;
use syn::Ident;

pub(super) async fn generate_blocks(datamap: &DataMap, args: &CliArgs) -> anyhow::Result<()> {
    // Map block names to block types.
    let mut block_set: HashMap<&str, BlockType> = HashMap::new();
    for data in datamap.version_data.values() {
        for block in data.blocks.iter() {
            let blocktype = if block.states() == 1 { BlockType::Unit } else { BlockType::U16 };

            if let Some(block) = block_set.get_mut(block.name.as_str()) {
                // If the existing block is a unit and the new block is a u16, update the block.
                if matches!((*block, blocktype), (BlockType::Unit, BlockType::U16)) {
                    *block = blocktype;
                }
            } else {
                // Insert the block if it doesn't exist.
                block_set.insert(&block.name, blocktype);
            }
        }
    }

    // Access the block names in sorted order.
    let mut block_list: Vec<_> = block_set.keys().collect();
    block_list.sort_unstable();

    // Generate the block tokens.
    let mut block_tokens = String::new();
    for (block_name, block_type) in
        block_list.into_iter().map(|&block_name| (block_name, block_set[block_name]))
    {
        // Format the block name.
        let block_name = block_name.replace(['\''], "_").to_case(Case::Pascal);
        let block_name = Ident::new(&block_name, Span::call_site());

        // Generate the block tokens based on the block type.
        block_tokens.push_str(
            match block_type {
                BlockType::Unit => format!("    pub struct {block_name};"),
                BlockType::U16 => format!("    pub struct {block_name}(pub(super) u16);"),
            }
            .as_str(),
        );
        block_tokens.push('\n');
    }

    // Wrap the tokens in a macro and generate the output.
    let content = format!(
r"//! Generated blocks for all 
//! [`Versions`](froglight_protocol::traits::Version).
//!
//! @generated by 'TODO' 
#![allow(missing_docs)]

froglight_macros::impl_generated_blocks! {{
{block_tokens}
}}");

    // Write the output to a file.
    let block_path = args.dir.join("crates/froglight-block/src/generated/block.rs");
    if !block_path.exists() {
        tracing::warn!("BlockGenerator: Creating file \"{}\"", block_path.display());
        tokio::fs::create_dir_all(block_path.parent().unwrap()).await?;
    }
    tokio::fs::write(block_path, &content).await?;

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BlockType {
    Unit,
    U16,
}

pub(super) async fn generate_block_impls(
    datamap: &DataMap,
    overrides: &[(String, String)],
    args: &CliArgs,
) -> anyhow::Result<()> {
    for (version, data) in &datamap.version_data {
        generate_block_version_impl(version, &data.blocks, overrides, args).await?;
    }
    Ok(())
}

async fn generate_block_version_impl(
    version: &Version,
    blocks: &VersionBlocks,
    overrides: &[(String, String)],
    args: &CliArgs,
) -> anyhow::Result<()> {
    let version_name = format!("V{}", version.to_long_string().replace(['.'], "_"));
    let module_name = version_name.to_ascii_lowercase();
    let mut content = format!(
        r"//! Generated block implementations for [`{version_name}`].
//!
//! @generated by 'TODO'
#![allow(
    missing_docs,
    clippy::cast_possible_truncation,
    clippy::unreadable_literal,
    clippy::wildcard_imports
)]

use froglight_protocol::versions::{module_name}::{version_name};

use super::{{attribute::*, block::*}};
use crate::{{BlockState, BlockStateExt}};

froglight_macros::impl_block_traits! {{
    {version_name} => {{
"
    );

    for block in blocks.iter() {
        let block_name = block.name.replace(['\''], "_").to_case(Case::Pascal);

        let resource_key = block.name.as_str();
        let material = block.material.as_str();
        let diggable = block.diggable;
        let hardness = block.hardness;
        let resistance = block.resistance;
        let transparent = block.transparent;
        let emit_light = block.emit_light;
        let bounding_box = block.bounding_box.as_str();

        if block.states.is_empty() {
            content.push_str(&format!(
r#"        {block_name} => ["minecraft:{resource_key}", "minecraft:{material}", {diggable}, {hardness}f32, {resistance}f32, {transparent}, {emit_light}u8, "minecraft:{bounding_box}"],
"#));
        } else {
            let default = block.default_state - block.min_state_id;

            let mut attributes = String::new();
            for (index, state) in block.states.iter().enumerate() {
                let mut field_type = BlockGenerator::attribute_item_name(state);
                if let Some((_, new_ident)) = overrides.iter().find(|(old, _)| *old == field_type) {
                    field_type = new_ident.clone();
                }
    
                attributes.push_str(&field_type);
                if index < block.states.len().saturating_sub(1) {
                    attributes.push_str(", ");
                }
            }

            content.push_str(&format!(
r#"        {block_name} => ({attributes}),
                ["minecraft:{resource_key}", "minecraft:{material}", {diggable}, {hardness}f32, {resistance}f32, {transparent}, {emit_light}u8, "minecraft:{bounding_box}", {default}],
"#));
        }
    }

    // Close the macro.
    content.push_str("    }\n}\n");

    // Write the output to a file.
    let file_name = format!("v{}.rs", version.to_long_string().replace(['.'], "_"));
    let impl_path = args.dir.join("crates/froglight-block/src/generated").join(file_name);
    if !impl_path.exists() {
        tracing::warn!("BlockGenerator: Creating file \"{}\"", impl_path.display());
        tokio::fs::create_dir_all(impl_path.parent().unwrap()).await?;
    }
    tokio::fs::write(impl_path, &content).await?;

    Ok(())
}
