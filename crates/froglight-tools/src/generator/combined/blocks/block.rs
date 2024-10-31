use std::path::Path;

use anyhow::bail;
use convert_case::{Case, Casing};
use froglight_extract::bundle::ExtractBundle;
use hashbrown::HashMap;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{bundle::GenerateBundle, consts::GENERATE_NOTICE, helpers::format_file};

pub(super) async fn generate_blocks(
    blck_path: &Path,
    _generate: &GenerateBundle<'_>,
    extract: &ExtractBundle,
) -> anyhow::Result<()> {
    // Create the block list
    let block_list = Block::create_list(extract)?;

    let mut block_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(blck_path)
        .await?;

    // Write the docs and notice
    block_file.write_all(b"//! Generated blocks\n//!\n").await?;
    block_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
    block_file.write_all(b"\n\n").await?;

    // Import the block macro and attributes
    block_file.write_all(b"use froglight_macros::frog_create_blocks;\n").await?;
    block_file
        .write_all(
            b"#[allow(clippy::wildcard_imports)]\nuse crate::definitions::attributes::*;\n\n",
        )
        .await?;

    // Start the block macro
    block_file.write_all(b"frog_create_blocks! {\n").await?;

    let first = block_list.first().unwrap().clone();
    let namespace = first.raw_name.split(':').next().unwrap();
    block_file.write_all(format!("    \"{namespace}\",\n").as_bytes()).await?;

    // Write the blocks
    for Block { raw_name, name, fields } in block_list {
        let block_key = raw_name.trim_start_matches(namespace).trim_start_matches(':');

        // Write the block fields
        if fields.is_empty() {
            // Write the block struct
            block_file.write_all(format!("    {name} => \"{block_key}\",\n").as_bytes()).await?;
        } else {
            // Start the block struct
            block_file.write_all(format!("    {name} => \"{block_key}\" {{\n").as_bytes()).await?;

            // Write the block fields
            for (field_name, field_type) in fields {
                block_file
                    .write_all(format!("        pub {field_name}: {field_type},\n").as_bytes())
                    .await?;
            }

            // Finish the block struct
            block_file.write_all(b"    },\n").await?;
        }
    }

    // Finish the block macro
    block_file.write_all(b"}\n").await?;
    format_file(&mut block_file).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Block {
    /// The raw name of the block.
    ///
    /// For example, `minecraft:grass_block`.
    pub(crate) raw_name: String,

    /// The name of the block.
    ///
    /// For example, `GrassBlock`.
    pub(crate) name: String,

    /// The attribute fields of the block.
    ///
    /// For example, `{}`, `{"snowy": "SnowyAttribute"}`, etc.
    pub(crate) fields: HashMap<String, String>,
}

impl Block {
    /// Create a list of blocks from the [`ExtractBundle`].
    pub(crate) fn create_list(extract: &ExtractBundle) -> anyhow::Result<Vec<Self>> {
        let mut block_list: Vec<Block> = Vec::new();

        let block_data = extract.output["blocks"].as_object().unwrap();
        for (block_name, block_data) in block_data {
            let name = block_name.trim_start_matches("minecraft:").to_case(Case::Pascal);

            let mut fields = HashMap::new();

            if let Some(attribute_data) = block_data["properties"].as_object() {
                for (attr_name, attr_values) in attribute_data {
                    let attr_values = attr_values.as_array().unwrap();
                    let mut attr_values = attr_values
                        .iter()
                        .map(|v| v.as_str().unwrap().to_case(Case::Pascal))
                        .collect::<Vec<_>>();
                    attr_values.sort();

                    let attr_type_name = super::attribute::attribute_name(attr_name, &attr_values);

                    // Fix the `type` attribute name
                    let mut attr_name = attr_name.as_ref();
                    if attr_name == "type" {
                        attr_name = "kind";
                    }

                    // Error and exit if there are duplicate attribute names
                    if fields.insert(attr_name.to_string(), attr_type_name).is_some() {
                        bail!(
                            "Duplicate attribute name for block: \"{block_name}\" -> \"{attr_name}\""
                        );
                    }
                }
            }

            block_list.push(Block { raw_name: block_name.clone(), name, fields });
        }

        Ok(block_list)
    }
}
