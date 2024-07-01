use std::{collections::BTreeMap, path::Path};

use convert_case::{Case, Casing};
use froglight_extract::bundle::ExtractBundle;
use serde_json::{Map, Value};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{format_file, version_module_name, version_struct_name},
    modules::registries::generated::{AttributeType, Block},
};

/// Generate the block trait implementations for the given version.
///
/// This requires an intermediary macro to generate the actual block trait
/// implementations. Otherwise, the resulting file will be 300k+ lines and over
/// 16MB in size... And this runs for every protocol version :(
///
///
/// Example:
/// ```rust,ignore
/// frog_create_block_impls! {
///     // Version
///     V1_21_0,
///     Attributes => {
///         // Attribute Type :: [ Variants ]
///         Attrib: [One, Two, Three],
///     },
///     Blocks => {
///         // Block type with no attributes
///         EmptyBlock,
///         // Block type
///         Block {
///             // Default state, as an index into `permutations`
///             default: 1,
///             // Block fields
///             fields: {
///                 // Attribute index
///                 field_name: 0,
///             },
///             // Block attribute permutations
///             permutations: [
///                 // Attribute value index
///                 [0], [1], [2]
///             ],
///         },
///     },
/// }
/// ```
#[allow(clippy::too_many_lines)]
pub(super) async fn generate_blocks(
    blck_path: &Path,
    generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    // Create the attribute enum list
    let mut attrib_enum_list: Vec<(String, Vec<String>)> = Vec::new();

    let attrib_list = AttributeType::create_list(extract)?;
    for attrib in &attrib_list {
        match attrib {
            AttributeType::Boolean(name) => {
                attrib_enum_list
                    .push((name.to_string(), vec![String::from("True"), String::from("False")]));
            }
            AttributeType::Enum(name, variants) => {
                attrib_enum_list
                    .push((name.to_string(), variants.iter().map(String::to_string).collect()));
            }
            AttributeType::Range(name, min, max) => {
                attrib_enum_list
                    .push((name.to_string(), (*min..=*max).map(|v| v.to_string()).collect()));
            }
        }
    }

    // Get the block data
    let block_data = extract.output["blocks"].as_object().unwrap();

    let block_list = Block::create_list(extract)?;
    let data_list: Vec<BlockData> = block_list
        .iter()
        .map(|block| {
            let data = block_data[&block.raw_name].as_object().unwrap();
            BlockData::from_data(&attrib_enum_list, data)
        })
        .collect();

    // Generate the block implementations
    let module = version_module_name(&generate.version.base).to_string();
    let version = version_struct_name(&generate.version.base).to_string();

    // Open the file
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(blck_path)
        .await?;

    // Write the docs and notice
    file.write_all(
        format!("//! Generated block implementations for [`{version}`]\n//!\n").as_bytes(),
    )
    .await?;
    file.write_all(GENERATE_NOTICE.as_bytes()).await?;
    file.write_all(b"\n\n").await?;

    // Import the necessary modules
    file.write_all(
        format!(
            r#"
use froglight_macros::frog_create_block_impls;
use froglight_protocol::versions::{module}::{version};

#[allow(clippy::wildcard_imports)]
use crate::{{
    definitions::{{BlockExt, BlockStateResolver, BlockStorage, VanillaResolver}},
    registries::{{attributes::*, blocks::*}},
}};
"#
        )
        .as_bytes(),
    )
    .await?;

    // Start the macro
    file.write_all(b"\nfrog_create_block_impls! {\n").await?;

    // Write the version
    file.write_all(format!("    {version},\n").as_bytes()).await?;

    // Write the attributes
    file.write_all(b"    Attributes => {\n").await?;
    for (name, variants) in &attrib_enum_list {
        file.write_all(format!("        {name}: ").as_bytes()).await?;

        file.write_all(b"[").await?;
        for (index, variant) in variants.iter().enumerate() {
            if name.contains("Range") {
                file.write_all(format!("_{variant}").as_bytes()).await?;
                if index != variants.len() - 1 {
                    file.write_all(b", ").await?;
                }
            } else {
                file.write_all(variant.as_bytes()).await?;
                if index != variants.len() - 1 {
                    file.write_all(b", ").await?;
                }
            }
        }
        file.write_all(b"],\n").await?;
    }
    file.write_all(b"    },\n").await?;

    // Write the blocks
    file.write_all(b"    Blocks => {\n").await?;
    for (block, data) in block_list.iter().zip(data_list) {
        match data {
            BlockData::Default => {
                file.write_all(format!("        {},\n", block.name).as_bytes()).await?;
            }
            BlockData::Block { default, fields, permutations } => {
                file.write_all(format!("        {} {{\n", block.name).as_bytes()).await?;
                file.write_all(format!("            default: {default},\n").as_bytes()).await?;

                // Write the fields
                {
                    file.write_all(b"            fields: {").await?;
                    for (index, (field, attrib)) in fields.iter().enumerate() {
                        let mut field = field.as_str();
                        if field == "type" {
                            field = "kind";
                        }

                        file.write_all(format!(" {field}: {attrib}").as_bytes()).await?;
                        if index != fields.len() - 1 {
                            file.write_all(b",").await?;
                        }
                    }
                    file.write_all(b" },\n").await?;
                }

                // Write the permutations
                {
                    let field_len = fields.len();
                    let perm_len = permutations.len();

                    let mut line_length = 0;
                    let line_length_max = 80;

                    if field_len * permutations.len() * 3 < line_length_max {
                        // All permutations on one line
                        file.write_all(b"            permutations: [ ").await?;
                        for (index, values) in permutations {
                            file.write_all(b"[").await?;
                            for (index, value) in values.iter().enumerate() {
                                file.write_all(value.to_string().as_bytes()).await?;
                                if index != values.len() - 1 {
                                    file.write_all(b",").await?;
                                }
                            }
                            file.write_all(b"]").await?;

                            if usize::try_from(index).unwrap() != perm_len - 1 {
                                file.write_all(b", ").await?;
                            }
                        }
                        file.write_all(b" ],\n").await?;
                    } else {
                        // Permutations on multiple lines
                        file.write_all(b"            permutations: [\n").await?;
                        for (_index, values) in permutations {
                            if line_length == 0 {
                                file.write_all(b"                ").await?;
                            } else {
                                file.write_all(b", ").await?;
                            }

                            file.write_all(b"[").await?;
                            for (index, value) in values.iter().enumerate() {
                                file.write_all(value.to_string().as_bytes()).await?;
                                if index != field_len - 1 {
                                    file.write_all(b",").await?;
                                }
                            }

                            line_length += field_len * 3;
                            if line_length >= line_length_max {
                                file.write_all(b"],\n").await?;
                                line_length = 0;
                            } else {
                                file.write_all(b"]").await?;
                            }
                        }

                        if line_length != 0 {
                            file.write_all(b"\n").await?;
                        }
                        file.write_all(b"            ],\n").await?;
                    }
                }

                file.write_all(b"        },\n").await?;
            }
        }
    }
    file.write_all(b"    },\n").await?;

    // Finish the macro
    file.write_all(b"}\n").await?;

    format_file(&mut file).await
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockData {
    Default,
    Block {
        /// The default block state as an index into `permutations`
        default: i64,
        /// The block fields, with an attribute index
        fields: Vec<(String, usize)>,
        /// The block attribute permutations,
        /// with the attribute value indices
        permutations: BTreeMap<i64, Vec<usize>>,
    },
}

impl BlockData {
    fn from_data(attributes: &[(String, Vec<String>)], data: &Map<String, Value>) -> Self {
        let states = data["states"].as_array().unwrap();
        let states: Vec<_> = states.iter().map(|v| v.as_object().unwrap()).collect();

        // There is only one state, so it must be the default
        // and there must be no fields or permutations
        if states.len() == 1 {
            return BlockData::Default;
        }

        // Find the minimum protocol id
        let mut minimum = i64::MAX;
        for state in &states {
            let protocol_id = state["protocol_id"].as_i64().unwrap();
            if protocol_id < minimum {
                minimum = protocol_id;
            }
        }

        // Get the default state, block fields and, and state attributes
        let mut default = i64::MAX;
        let mut fields = Vec::new();
        let mut permutations = BTreeMap::new();

        for state in states {
            let protocol_id = state["protocol_id"].as_i64().unwrap();
            let relative_id = protocol_id - minimum;

            // Check if this is the default state
            if state.get("default").and_then(Value::as_bool).is_some_and(|b| b) {
                assert_eq!(default, i64::MAX, "Multiple default states found");
                default = relative_id;
            }

            let mut state_permutation = Vec::new();

            let state_attributes = state["properties"].as_object().unwrap();
            for (field_name, attrib_value) in state_attributes {
                let field_attrib =
                    AttributeType::create_from(field_name, &data["properties"][field_name]);

                let (attrib_index, (_, attrib_values)) = attributes
                    .iter()
                    .enumerate()
                    .find(|(_, (n, _))| n == field_attrib.name())
                    .unwrap();
                if fields.iter().all(|(n, _)| n != field_name) {
                    fields.push((field_name.clone(), attrib_index));
                }

                let attrib_value = attrib_value.as_str().unwrap();
                let attrib_value = attrib_value.to_case(Case::Pascal);

                let value_index = attrib_values.iter().position(|v| v == &attrib_value).unwrap();
                state_permutation.push(value_index);
            }

            permutations.insert(relative_id, state_permutation);
        }

        assert!(default >= 0, "Default permutation was negative?");
        assert_ne!(default, i64::MAX, "No default permutation found");
        Self::Block { default, fields, permutations }
    }
}
