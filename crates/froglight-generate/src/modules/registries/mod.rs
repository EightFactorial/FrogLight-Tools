use std::{future::Future, io::SeekFrom, path::Path, pin::Pin};

use anyhow::bail;
use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        builtin_json::{
            Blocks as ExtractBlocks, BuiltinJsonModule, Registries as ExtractRegistries,
        },
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tracing::{debug, warn};

use super::sealed::GenerateRequired;
use crate::{
    bundle::GenerateBundle,
    helpers::{update_file_modules, update_file_tag},
    modules::GenerateModule,
};

mod contents;
mod generated;
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
pub struct Registries;

impl Registries {
    /// The path to the `froglight-registry` src folder,
    /// relative to the root directory.
    const CRATE_SRC_PATH: &'static str = "crates/froglight-registry/src";
}

impl GenerateRequired for Registries {
    const REQUIRED: &'static [ExtractModules] = &[
        ExtractModules::BuiltinJson(BuiltinJsonModule::Blocks(ExtractBlocks)),
        ExtractModules::BuiltinJson(BuiltinJsonModule::Registries(ExtractRegistries)),
    ];
}

impl GenerateModule for Registries {
    /// Run the generation process.
    fn generate<'a>(
        &'a self,
        generate: &'a GenerateBundle<'_>,
        extract: &'a ExtractBundle<'_>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + Sync + 'a>> {
        Box::pin(async {
            // Get the path to the `froglight-registry` src folder.
            let src_path = generate.root_dir.join(Self::CRATE_SRC_PATH);
            if !src_path.exists() {
                bail!("Could not find `froglight-registry` src at \"{}\"!", src_path.display());
            }
            debug!("Found `froglight-registry` src at \"{}\"", src_path.display());

            // Get the path to the `registries` folder,
            // creating it if it doesn't exist.
            let reg_path = src_path.join(Self::REGISTRIES_PATH);
            if !reg_path.exists() {
                warn!("Creating missing `registries` directory at \"{}\"", reg_path.display());
                tokio::fs::create_dir(&reg_path).await?;
            }

            // Create the registries.
            Self::create_registries_contents(&reg_path, generate, extract).await?;

            // Update the `registries/mod.rs` file.
            Self::create_registries_mod(&reg_path.join("mod.rs")).await
        })
    }
}

impl Registries {
    /// The path to the `registries` folder,
    /// relative to the `src` folder.
    const REGISTRIES_PATH: &'static str = "registries";

    const REGISTRIES_DOCS: &'static str = "//! Generated registries and their implementations";

    /// Create the `registries/mod.rs` file.
    async fn create_registries_mod(path: &Path) -> anyhow::Result<()> {
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        // Write the docs
        mod_file.write_all(Self::REGISTRIES_DOCS.as_bytes()).await?;

        // Update the module list.
        update_file_modules(&mut mod_file, path, false, false).await?;
        mod_file.write_all(b"\npub use generated::*;\n").await?;

        // Update the build function.
        {
            let mut contents = String::new();
            mod_file.seek(SeekFrom::Start(0)).await?;
            mod_file.read_to_string(&mut contents).await?;

            let mut modules = Vec::new();
            for line in contents.lines() {
                if line.starts_with("mod") {
                    modules.push(line.split_whitespace().last().unwrap().trim_end_matches(';'));
                }
            }

            let mut build_modules = String::new();
            for version in modules {
                build_modules.push_str(&format!("{version}::build(app);\n"));
            }

            mod_file
                .write_all(
                    format!(
                        r#"
    #[doc(hidden)]
    pub(super) fn build(app: &mut bevy_app::App) {{
        {build_modules}
    }}"#
                    )
                    .as_bytes(),
                )
                .await?;
        }

        // Update the file tag.
        update_file_tag(&mut mod_file, path).await
    }
}
