use std::path::Path;

use froglight_extract::bundle::ExtractBundle;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{format_file, version_module_name, version_struct_name},
};

mod block;
mod registry;

pub(super) async fn create_versioned(
    ver_path: &Path,
    generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    let blck_path = ver_path.join("blocks.rs");
    block::generate_blocks(&blck_path, generate, extract).await?;

    let reg_path = ver_path.join("registries.rs");
    registry::generate_registries(&reg_path, generate, extract).await?;

    let mod_path = ver_path.join("mod.rs");
    let mut mod_file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(&mod_path)
        .await?;

    let version = version_struct_name(&generate.version.jar).to_string();
    let module = version_module_name(&generate.version.jar).to_string();

    // Write the docs and notice
    mod_file.write_all(format!("//! Generated registries for [`{version}`]\n").as_bytes()).await?;
    mod_file.write_all(format!("//!\n{GENERATE_NOTICE}\n\n").as_bytes()).await?;

    // Write the imports
    mod_file
        .write_all(format!("use froglight_protocol::versions::{module}::{version};\n\n").as_bytes())
        .await?;
    mod_file.write_all(b"use crate::definitions::BlockRegistry;\n\n").await?;

    // Write the modules
    mod_file.write_all(b"mod blocks;\n").await?;
    mod_file.write_all(b"mod registries;\n\n").await?;

    // Write the build function
    mod_file.write_all(format!("#[doc(hidden)]\npub(super) fn build(app: &mut bevy_app::App) {{ app.init_resource::<BlockRegistry<{version}>>(); }}\n").as_bytes()).await?;

    // Format the file
    format_file(&mut mod_file).await
}
