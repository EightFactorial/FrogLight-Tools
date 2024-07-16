use std::{cmp::Ordering, future::Future, io::SeekFrom, path::Path, pin::Pin};

use anyhow::bail;
use froglight_definitions::MinecraftVersion;
use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        builtin_json::{Blocks as ExtractBlocks, BuiltinJsonModule},
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tracing::{debug, warn};

use super::sealed::GenerateRequired;
use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{format_file, update_file_modules, version_module_name},
    modules::GenerateModule,
};

mod attribute;
mod block;
mod version;

/// A module that generates blocks and registries.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize_unit_struct,
    Deserialize_unit_struct,
)]
pub struct Blocks;

impl Blocks {
    /// The path to the `froglight-block` src folder,
    /// relative to the root directory.
    const CRATE_SRC_PATH: &'static str = "crates/froglight-block/src";

    /// The path to the `definitions` folder,
    /// relative to the src folder.
    const DEF_SRC_PATH: &'static str = "definitions";
}

impl GenerateRequired for Blocks {
    const REQUIRED: &'static [ExtractModules] =
        &[ExtractModules::BuiltinJson(BuiltinJsonModule::Blocks(ExtractBlocks))];
}

impl GenerateModule for Blocks {
    /// Run the generation process.
    fn generate<'a>(
        &'a self,
        generate: &'a GenerateBundle<'_>,
        extract: &'a ExtractBundle<'_>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + Sync + 'a>> {
        Box::pin(async {
            // Get the path to the `froglight-block` src folder.
            let src_path = generate.root_dir.join(Self::CRATE_SRC_PATH);
            if !src_path.exists() {
                bail!("Could not find `froglight-block` src at \"{}\"!", src_path.display());
            }
            debug!("Found `froglight-block` src at \"{}\"", src_path.display());

            // Get the path to the `definitions` folder,
            // creating it if it doesn't exist.
            let def_path = src_path.join(Self::DEF_SRC_PATH);
            if !def_path.exists() {
                warn!("Creating missing `definitions` directory at \"{}\"", def_path.display());
                tokio::fs::create_dir(&def_path).await?;
            }

            // Check if the blocks and block attributes should be generated
            let mod_path = def_path.join("mod.rs");
            let gen = should_generate(&mod_path, generate, extract).await?;

            // Create the block attributes
            if gen {
                let attr_path = def_path.join("attributes.rs");
                debug!("Generating block attributes: \"{}\"", &generate.version.base);
                attribute::generate_attributes(&attr_path, generate, extract).await?;
            }

            // Crate the blocks
            if gen {
                let blck_path = def_path.join("blocks.rs");
                debug!("Generating blocks: \"{}\"", &generate.version.base);
                block::generate_blocks(&blck_path, generate, extract).await?;
            }

            // Create versioned implementations of the registries
            {
                let ver_mod_name = version_module_name(&generate.version.jar).to_string();
                let mut ver_path = def_path.join(ver_mod_name);
                ver_path.set_extension("rs");
                version::generate_blocks(&ver_path, generate, extract).await?;
            }

            // Update the `mod.rs` file
            {
                let mut mod_file = tokio::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&mod_path)
                    .await?;

                // Write the docs and notice
                mod_file.write_all(b"//! Generated blocks and block attributes\n//!\n").await?;
                mod_file
                    .write_all(
                        format!("//! Template: {}\n//!\n", generate.version.base.to_long_string())
                            .as_bytes(),
                    )
                    .await?;
                mod_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
                mod_file.write_all(b"\n").await?;

                // Allow missing documentation
                mod_file.write_all(b"#![allow(missing_docs)]\n\n").await?;

                // Add modules
                update_file_modules(&mut mod_file, &mod_path, true, false).await?;

                // Collect the version modules, make only `blocks` and `attributes` public
                let mut modules = Vec::new();
                {
                    mod_file.seek(SeekFrom::Start(0)).await?;
                    let mut contents = String::new();
                    mod_file.read_to_string(&mut contents).await?;

                    mod_file.seek(SeekFrom::Start(0)).await?;
                    mod_file.set_len(0).await?;

                    for line in contents.lines() {
                        if let Some(stripped) = line.strip_prefix("pub mod ") {
                            if matches!(stripped, "attributes;" | "blocks;") {
                                mod_file.write_all(line.as_bytes()).await?;
                            } else {
                                modules.push(stripped.trim_end_matches(';').to_string());

                                mod_file.write_all(b"mod ").await?;
                                mod_file.write_all(stripped.as_bytes()).await?;
                            }
                        } else {
                            mod_file.write_all(line.as_bytes()).await?;
                        }
                        mod_file.write_all(b"\n").await?;
                    }
                    mod_file.write_all(b"\n").await?;
                }

                // Add the build function
                {
                    mod_file
                        .write_all(
                            br#"#[doc(hidden)]
#[cfg(feature = "bevy")]
pub(super) fn build(app: &mut bevy_app::App) {
    #[cfg(feature = "reflect")]
    { app.register_type::<blocks::Blocks>(); }

"#,
                        )
                        .await?;

                    for module in &modules {
                        let full_path = format!(
                            "froglight_protocol::versions::{module}::{}",
                            module.to_ascii_uppercase()
                        );
                        mod_file.write_all(format!("    app.init_resource::<crate::BlockRegistry<{full_path}>>();\n").as_bytes()).await?;
                    }

                    mod_file.write_all(b"}\n\n").await?;
                }

                // Format the file
                format_file(&mut mod_file).await?;
            }

            Ok(())
        })
    }
}

/// Returns `true` if the registries should be generated.
async fn should_generate(
    path: &Path,
    generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<bool> {
    if !path.exists() {
        return Ok(true);
    }

    let contents = tokio::fs::read_to_string(path).await?;
    for line in contents.lines().filter(|&l| l.starts_with("//!")) {
        if let Some(stripped) = line.strip_prefix("//! Template: ") {
            let generated = MinecraftVersion::from(stripped);
            if let Some(cmp) = extract.manifests.version.compare(&generated, &generate.version.base)
            {
                return Ok(cmp != Ordering::Greater);
            }
        }
    }

    warn!(
        "Unable to determine version, generating blocks and attributes for: \"{}\"",
        generate.version.base
    );
    Ok(true)
}
