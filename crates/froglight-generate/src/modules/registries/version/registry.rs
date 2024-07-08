use std::path::Path;

use froglight_extract::bundle::ExtractBundle;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{format_file, version_module_name, version_struct_name},
    modules::registries::generated::Registry,
};

pub(super) async fn generate_registries(
    reg_path: &Path,
    generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    let registry_data = extract.output["registries"].as_object().unwrap();
    let registries: Vec<Registry> = registry_data
        .iter()
        .map(|(name, data)| {
            let data = data.as_object().unwrap();
            Registry::from_data(name, data)
        })
        .collect();

    let mut reg_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&reg_path)
        .await?;

    let version = version_struct_name(&generate.version.base).to_string();
    let module = version_module_name(&generate.version.base).to_string();

    // Write the docs and notice
    reg_file
        .write_all(
            format!("//! Generated registry implementations for [`{version}`]\n//!\n").as_bytes(),
        )
        .await?;
    reg_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
    reg_file.write_all(b"\n\n").await?;

    // Write the imports
    reg_file.write_all(b"use froglight_macros::frog_create_registry_impls;\n").await?;
    reg_file
        .write_all(format!("use froglight_protocol::versions::{module}::{version};\n\n").as_bytes())
        .await?;

    reg_file.write_all(b"#[allow(clippy::wildcard_imports)]\n").await?;
    reg_file
        .write_all(b"use crate::{definitions::ConvertId, registries::registries::*};\n\n")
        .await?;

    // Start the registry implementation macro
    reg_file.write_all(b"frog_create_registry_impls! {\n").await?;

    // Write the version
    reg_file.write_all(format!("    {version},\n").as_bytes()).await?;

    // List all the registries
    for registry in &registries {
        // Write the registry name
        reg_file.write_all(format!("    {} {{ \n        ", registry.name).as_bytes()).await?;
        // Write the registry entries
        for (index, (_, name)) in registry.entries.iter().enumerate() {
            reg_file.write_all(name.as_bytes()).await?;

            if index != registry.entries.len() - 1 {
                reg_file.write_all(b", ").await?;
            }
        }
        // Close the registry
        reg_file.write_all(b"\n    },\n").await?;
    }

    // Finish the registry implementation macro
    reg_file.write_all(b"}\n").await?;

    format_file(&mut reg_file).await
}
