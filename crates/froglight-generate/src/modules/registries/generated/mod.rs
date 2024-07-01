use std::{cmp::Ordering, io::SeekFrom, path::Path, str::FromStr};

use froglight_definitions::MinecraftVersion;
use froglight_extract::bundle::ExtractBundle;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::debug;

use crate::{bundle::GenerateBundle, helpers::update_tag};

mod attribute;
pub(super) use attribute::AttributeType;

mod block;
pub(super) use block::Block;

mod registry;

pub(super) async fn create_generated(
    gen_path: &Path,
    generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    let mod_path = gen_path.join("mod.rs");
    if let Some(version) = existing_version(&mod_path).await {
        if extract.manifests.version.compare(&version, &generate.version.base)
            == Some(Ordering::Less)
        {
            debug!(
                "Skipping generating registries for \"{version}\", older than \"{}\"",
                generate.version.base
            );
            return Ok(());
        }
        debug!("Generating registries for \"{version}\"");
    }

    // Generate block attributes
    let attr_path = gen_path.join("attributes.rs");
    attribute::generate_attributes(&attr_path, generate, extract).await?;

    // Generate blocks
    let blck_path = gen_path.join("blocks.rs");
    block::generate_blocks(&blck_path, generate, extract).await?;

    // Generate registries
    let reg_path = gen_path.join("registries");
    registry::generate_registries(&reg_path, generate, extract).await?;

    // Update the version and tag
    update_version(&mod_path, &generate.version.base).await?;
    update_tag(&mod_path).await
}

/// Read the existing version from the file.
async fn existing_version(mod_path: &Path) -> Option<MinecraftVersion> {
    let contents = tokio::fs::read_to_string(mod_path).await.ok()?;

    let mut first_line = contents.lines().next()?;
    first_line = first_line.trim_start_matches("//! Version: `").trim_end_matches('`');

    let version = MinecraftVersion::from_str(first_line).ok()?;
    if let MinecraftVersion::Other(..) = &version {
        None
    } else {
        Some(version)
    }
}

/// Update the version in the file.
async fn update_version(mod_path: &Path, version: &MinecraftVersion) -> anyhow::Result<()> {
    let mut mod_file = tokio::fs::OpenOptions::new().read(true).write(true).open(mod_path).await?;

    // Read the file contents
    let mut contents = String::new();
    mod_file.read_to_string(&mut contents).await?;

    // Clear the file
    mod_file.seek(SeekFrom::Start(0)).await?;
    mod_file.set_len(0).await?;

    // Write the contents, but with the new version
    for line in contents.lines() {
        if line.starts_with("//! Version: `") {
            mod_file
                .write_all(format!("//! Version: `{}`\n", version.to_long_string()).as_bytes())
                .await?;
        } else {
            mod_file.write_all(line.as_bytes()).await?;
            mod_file.write_all(b"\n").await?;
        }
    }

    Ok(())
}
