use std::{future::Future, pin::Pin};

use anyhow::bail;
use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        builtin_json::{BuiltinJsonModule, Registries as ExtractRegistries},
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tokio::io::AsyncWriteExt;
use tracing::{debug, warn};

use super::sealed::GenerateRequired;
use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{format_file, update_file_modules, version_module_name},
    modules::GenerateModule,
};

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

    /// The path to the `definitions` folder,
    /// relative to the src folder.
    const DEF_SRC_PATH: &'static str = "definitions";
}

impl GenerateRequired for Registries {
    const REQUIRED: &'static [ExtractModules] =
        &[ExtractModules::BuiltinJson(BuiltinJsonModule::Registries(ExtractRegistries))];
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

            // Get the path to the `defitions` folder,
            // creating it if it doesn't exist.
            let def_path = src_path.join(Self::DEF_SRC_PATH);
            if !def_path.exists() {
                warn!("Creating missing `defintions` directory at \"{}\"", def_path.display());
                tokio::fs::create_dir(&def_path).await?;
            }

            // Create the generated registries
            {
                let gen_path = def_path.join("generated");
                generated::generate_registries(&gen_path, generate, extract).await?;
            }

            // Create versioned implementations of the registries
            {
                let ver_mod_name = version_module_name(&generate.version.jar).to_string();
                let mut ver_path = def_path.join(ver_mod_name);
                ver_path.set_extension("rs");
                version::generate_registries(&ver_path, generate, extract).await?;
            }

            // Update the `mod.rs` file
            {
                let mod_path = def_path.join("mod.rs");
                let mut mod_file = tokio::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&mod_path)
                    .await?;

                // Write the docs and notice
                mod_file.write_all(b"//! Generated registry implementations\n//!\n").await?;
                mod_file.write_all(GENERATE_NOTICE.as_bytes()).await?;
                mod_file.write_all(b"\n").await?;

                // Allow missing documentation
                mod_file.write_all(b"#![allow(missing_docs)]\n\n").await?;

                // Add modules
                update_file_modules(&mut mod_file, &mod_path, false, false).await?;

                // Reexport the generated registries
                mod_file.write_all(b"\npub use generated::*;\n\n").await?;

                // Add the build function
                mod_file
                    .write_all(
                        br"#[doc(hidden)]
pub(super) fn build(app: &mut bevy_app::App) { generated::build(app); }
",
                    )
                    .await?;

                // Format the file
                format_file(&mut mod_file).await?;
            }

            Ok(())
        })
    }
}
