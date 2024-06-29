use std::path::Path;

use convert_case::{Case, Casing};
use froglight_extract::bundle::ExtractBundle;
use serde_json::{Map, Value};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::trace;

use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{format_file, update_file_modules},
};

pub(super) async fn generate_registries(
    reg_path: &Path,
    _generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    // Delete and recreate the registries directory
    if reg_path.exists() {
        tokio::fs::remove_dir_all(&reg_path).await?;
    }
    tokio::fs::create_dir_all(&reg_path).await?;

    // Generate the registries
    let mut generated_registries = Vec::new();

    let registry_data = extract.output["registries"].as_object().unwrap();
    for (reg_name, reg_data) in registry_data {
        let reg_data = reg_data.as_object().unwrap();
        generated_registries.push(generate_registry(reg_name, reg_data, reg_path).await?);
    }

    // Create the mod file
    let mod_path = reg_path.join("mod.rs");
    let mut mod_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&mod_path)
        .await?;

    // Write the docs and notice
    mod_file.write_all(b"//! Generated registries\n//!\n").await?;
    mod_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
    mod_file.write_all(b"\n\n").await?;

    // Update modules and reexport registries
    update_file_modules(&mut mod_file, &mod_path, false, true).await?;

    // Create the `build` function
    mod_file
        .write_all(b"\n#[doc(hidden)]\npub(super) fn build(app: &mut bevy_app::App) {\n")
        .await?;
    for reg in &generated_registries {
        mod_file.write_all(format!("    app.register_type::<{reg}>();\n").as_bytes()).await?;
    }
    mod_file.write_all(b"}\n").await?;
    format_file(&mut mod_file).await
}

async fn generate_registry(
    name: &str,
    data: &Map<String, Value>,
    reg_path: &Path,
) -> anyhow::Result<String> {
    let file_name = name.trim_start_matches("minecraft:").replace('/', "_").to_case(Case::Snake);
    let registry_path = reg_path.join(format!("{file_name}.rs"));

    trace!("Generating registry \"{name}\" at \"{}\"", registry_path.display());

    let mut registry_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&registry_path)
        .await?;

    // Write the notice
    registry_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
    registry_file.write_all(b"\n\n").await?;

    // Import macros
    registry_file
        .write_all(b"use froglight_macros::FrogRegistry;\nuse bevy_reflect::Reflect;\n\n")
        .await?;

    // Create the registry enum identity
    let mut key_name =
        name.trim_start_matches("minecraft:").replace([':', '/', '.'], "_").to_case(Case::Pascal);
    key_name.push_str("Key");

    let default_entry = data.get("default").and_then(|v| v.as_str());

    // Write the registry derive macro
    registry_file
        .write_all(if default_entry.is_some() {
            b"#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect, FrogRegistry)]"
        } else {
            b"#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, FrogRegistry)]"
        })
        .await?;

    // Start the registry enum
    registry_file.write_all(format!("\npub enum {key_name} {{\n").as_bytes()).await?;

    // Sort the entries by protocol ID
    let entries: &Map<String, Value> = data["entries"].as_object().unwrap();
    let mut entries: Vec<(&String, i64)> =
        entries.into_iter().map(|(k, v)| (k, v["protocol_id"].as_i64().unwrap())).collect();
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    // Write the registry values
    for (entry_name, _) in entries {
        let entry_ident = entry_name
            .trim_start_matches("minecraft:")
            .replace([':', '/', '.'], "_")
            .to_case(Case::Pascal);

        // Write the key attribute
        registry_file.write_all(format!(r#"    #[frog(key = "{entry_name}")]"#).as_bytes()).await?;
        registry_file.write_all(b"\n").await?;

        // Mark the default entry
        if default_entry.is_some_and(|n| n == entry_name) {
            registry_file.write_all(b"    #[default]\n").await?;
        }

        // Write the entry
        registry_file.write_all(format!("    {entry_ident},\n").as_bytes()).await?;
    }

    // Finish the registry enum
    registry_file.write_all(b"}\n").await?;

    // Format the registry file
    format_file(&mut registry_file).await?;

    // Return the registry enum name
    Ok(key_name)
}
