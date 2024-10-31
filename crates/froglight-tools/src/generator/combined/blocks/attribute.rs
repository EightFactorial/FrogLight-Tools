use std::path::Path;

use anyhow::bail;
use convert_case::{Case, Casing};
use froglight_extract::bundle::ExtractBundle;
use serde_json::Value;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::debug;

use crate::{bundle::GenerateBundle, consts::GENERATE_NOTICE, helpers::format_file};

pub(super) async fn generate_attributes(
    attr_path: &Path,
    _generate: &GenerateBundle<'_>,
    extract: &ExtractBundle,
) -> anyhow::Result<()> {
    let mut block_attributes = Vec::new();

    let block_data = extract.output["blocks"].as_object().unwrap();
    for block in block_data.values() {
        let Some(attributes) = block["properties"].as_object() else {
            continue;
        };

        for (attr_name, attr_values) in attributes {
            block_attributes.push(AttributeType::create_from(attr_name, attr_values));
        }
    }

    // Sort and deduplicate the attributes
    block_attributes.sort_by(|a, b| a.name().cmp(b.name()));
    block_attributes.dedup();

    // Error if there are any duplicate attribute names
    for (i, attr) in block_attributes.iter().enumerate() {
        for other in block_attributes.iter().skip(i + 1) {
            if attr.name() == other.name() {
                bail!("Duplicate attribute name: \"{}\"", attr.name());
            }
        }
    }

    // Write the attributes to the file
    let mut attr_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(attr_path)
        .await?;

    // Write the docs and notice
    attr_file.write_all(b"//! Generated block attributes\n//!\n").await?;
    attr_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
    attr_file.write_all(b"\n\n").await?;

    // Import the attribute macro
    attr_file.write_all(b"use froglight_macros::frog_create_attributes;\n\n").await?;

    // Start the attribute macro
    attr_file.write_all(b"frog_create_attributes! {\n").await?;

    // Write the attributes
    for attr in block_attributes {
        match attr {
            AttributeType::Boolean(name) => {
                // Write the boolean attribute
                attr_file.write_all(format!("    {name}(pub bool),\n").as_bytes()).await?;
            }
            AttributeType::Enum(name, values) => {
                // Start the enum attribute
                attr_file.write_all(format!("    {name} {{\n        ").as_bytes()).await?;

                // Write the values
                let values_len = values.len();
                for (index, value) in values.into_iter().enumerate() {
                    attr_file.write_all(value.as_bytes()).await?;

                    if index != values_len - 1 {
                        attr_file.write_all(b", ").await?;
                    }
                }

                // Finish the enum attribute
                attr_file.write_all(b"\n    },\n").await?;
            }
            AttributeType::Range(name, min, max) => {
                // Start the range attribute
                attr_file.write_all(format!("    {name} {{\n        ").as_bytes()).await?;

                // Write the values
                for value in min..=max {
                    attr_file.write_all(format!("_{value}").as_bytes()).await?;

                    if value != max {
                        attr_file.write_all(b", ").await?;
                    }
                }

                // Finish the range attribute
                attr_file.write_all(b"\n    },\n").await?;
            }
        }
    }

    // Finish the attribute macro
    attr_file.write_all(b"}\n").await?;
    format_file(&mut attr_file).await
}

/// Generate the attribute name.
///
/// Allows for formatting special cases.
pub(crate) fn attribute_name(name: &str, values: &[String]) -> String {
    let attr_name = name.to_case(Case::Pascal);
    if matches!(name, "north" | "south" | "east" | "west" | "up" | "down") {
        let mut joined = values.join("");
        if joined == "FalseTrue" {
            joined = String::from("Boolean");
        }

        // Direction Facing Attributes
        format!("{attr_name}Facing{joined}Attribute")
    } else if matches!(
        name,
        "instrument"
            | "attachment"
            | "orientation"
            | "sculk_sensor_phase"
            | "shapeinner"
            | "thickness"
            | "trial_spawner_state"
            | "vault_state"
    ) {
        // Special cases for very long attribute names
        format!("{attr_name}Attribute")
    } else if values[0].parse::<i32>().is_ok() {
        let mut min = values[0].parse::<i32>().unwrap();
        let mut max = values[0].parse::<i32>().unwrap();

        // Get the min and max values
        for value in values.iter().skip(1) {
            let value = value.parse::<i32>().unwrap();
            if value < min {
                min = value;
            } else if value > max {
                max = value;
            }
        }

        format!("{attr_name}{min}{max}RangeAttribute")
    } else {
        let mut values = values.to_vec();
        values.sort();

        let mut joined = values.join("");
        if joined == "FalseTrue" {
            joined = String::from("Boolean");
        }

        // Join all possible values into the attribute name
        format!("{attr_name}{joined}Attribute")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AttributeType {
    Boolean(String),
    Enum(String, Vec<String>),
    Range(String, i32, i32),
}

impl AttributeType {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Boolean(name) | Self::Enum(name, ..) | Self::Range(name, ..) => name,
        }
    }

    /// Create a list of attributes from the [`ExtractBundle`].
    pub(crate) fn create_list(extract: &ExtractBundle) -> anyhow::Result<Vec<Self>> {
        let mut block_attributes = Vec::new();

        let block_data = extract.output["blocks"].as_object().unwrap();
        for block in block_data.values() {
            let Some(attributes) = block["properties"].as_object() else {
                continue;
            };

            for (attr_name, attr_values) in attributes {
                block_attributes.push(AttributeType::create_from(attr_name, attr_values));
            }
        }

        // Sort and deduplicate the attributes
        block_attributes.sort_by(|a, b| a.name().cmp(b.name()));
        block_attributes.dedup();

        // Error if there are any duplicate attribute names
        for (i, attr) in block_attributes.iter().enumerate() {
            for other in block_attributes.iter().skip(i + 1) {
                if attr.name() == other.name() {
                    bail!("Duplicate attribute name: \"{}\"", attr.name());
                }
            }
        }

        Ok(block_attributes)
    }

    /// Create an attribute from the attribute name and values.
    pub(crate) fn create_from(attr_name: &str, attr_values: &Value) -> Self {
        let attr_values = attr_values.as_array().unwrap();
        let mut attr_values = attr_values
            .iter()
            .map(|v| v.as_str().unwrap().to_case(Case::Pascal))
            .collect::<Vec<_>>();
        attr_values.sort();

        let new_attr_name = attribute_name(attr_name, &attr_values);

        // Warn about long attribute names
        if new_attr_name.len() >= 40 && attr_name != "shape" {
            debug!("Consider shortening the attribute: \"{attr_name}\" -> \"{new_attr_name}\"");
        }

        if attr_values.len() == 2
            && attr_values.contains(&String::from("True"))
            && attr_values.contains(&String::from("False"))
        {
            AttributeType::Boolean(new_attr_name)
        } else if !attr_name.contains("range") && new_attr_name.ends_with("RangeAttribute") {
            let mut min = attr_values[0].parse::<i32>().unwrap();
            let mut max = attr_values[0].parse::<i32>().unwrap();

            // Find the min and max values
            for value in attr_values.iter().skip(1) {
                let value = value.parse::<i32>().unwrap();
                if value < min {
                    min = value;
                } else if value > max {
                    max = value;
                }
            }

            AttributeType::Range(new_attr_name, min, max)
        } else {
            AttributeType::Enum(new_attr_name, attr_values)
        }
    }
}
