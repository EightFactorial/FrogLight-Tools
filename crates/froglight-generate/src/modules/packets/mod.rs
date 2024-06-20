use std::path::Path;

use anyhow::bail;
use froglight_extract::{
    bundle::ExtractBundle,
    sources::{
        builtin_json::{BuiltinJsonModule, Version as ExtractVersion},
        bytecode::{BytecodeModule, Packets as ExtractPackets},
        Modules as ExtractModules,
    },
};
use serde_unit_struct::{Deserialize_unit_struct, Serialize_unit_struct};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::{debug, warn};

use super::sealed::GenerateRequired;
use crate::{
    bundle::GenerateBundle,
    helpers::{update_file_modules, update_file_tag},
    modules::GenerateModule,
};

mod version;

/// A module that generates states and packets.
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
pub struct Packets;

impl Packets {
    /// The path to the `froglight-protocol` src folder,
    /// relative to the root directory.
    const CRATE_SRC_PATH: &'static str = "crates/froglight-protocol/src";

    /// The path to the `versions` folder,
    /// relative to the `src` folder.
    const VERSIONS_PATH: &'static str = "versions";

    const VERSIONS_DOCS: &'static str =
        "//! Protocol versions and version-dependent structs and enums";

    /// Update the `versions/mod.rs` file.
    async fn update_version_module(path: &Path) -> anyhow::Result<()> {
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        // Write the docs, update the module list, and update the tag
        mod_file.write_all(Self::VERSIONS_DOCS.as_bytes()).await?;
        update_file_modules(&mut mod_file, path, true, false).await?;
        update_file_tag(&mut mod_file, path).await
    }
}

impl GenerateRequired for Packets {
    const REQUIRED: &'static [ExtractModules] = &[
        ExtractModules::BuiltinJson(BuiltinJsonModule::Version(ExtractVersion)),
        ExtractModules::Bytecode(BytecodeModule::Packets(ExtractPackets)),
    ];
}

impl GenerateModule for Packets {
    /// Run the generation process.
    async fn generate(
        &self,
        generate: &GenerateBundle<'_>,
        extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        // Get the path to the `froglight-protocol` src folder.
        let src_path = generate.root_dir.join(Self::CRATE_SRC_PATH);
        if !src_path.exists() {
            bail!("Could not find `froglight-protocol` src at \"{}\"!", src_path.display());
        }
        debug!("Found `froglight-protocol` src at \"{}\"", src_path.display());

        // Get the path to the `versions` folder,
        // creating it if it doesn't exist.
        let ver_path = src_path.join(Self::VERSIONS_PATH);
        if !ver_path.exists() {
            warn!("Creating missing `versions` directory at \"{}\"", ver_path.display());
            tokio::fs::create_dir(&ver_path).await?;
        }

        // Create the versioned packets.
        Self::create_version(&ver_path, generate, extract).await?;

        // Update the `versions/mod.rs` file.
        Self::update_version_module(&ver_path.join("mod.rs")).await
    }
}
