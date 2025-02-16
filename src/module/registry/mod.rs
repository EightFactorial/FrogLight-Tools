use std::{path::Path, sync::Arc};

use froglight_dependency::{
    container::DependencyContainer, dependency::mojang::VersionManifest, version::Version,
};
use froglight_extract::module::ExtractModule;
use structure::{DataStructures, VersionStructures};

use super::ToolConfig;

mod structure;

#[derive(ExtractModule)]
#[module(function = Registry::generate)]
pub(crate) struct Registry;

impl Registry {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-registry");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-registry\" at \"{}\"", directory.display());
        }

        let mut versions: Vec<_> = deps.get::<ToolConfig>().unwrap().versions.clone();

        // Sort versions using the manifest.
        let manifest = deps.get_or_retrieve::<VersionManifest>().await?;
        versions.sort_by(|a, b| manifest.compare(a, b).unwrap());

        // Generate registries using the current and previous versions.
        let previous = versions.iter().position(|v| v == version).and_then(|i| versions.get(i - 1));
        Self::generate_registries(version, previous, deps, &directory)
            .await
            .map_err(|e| anyhow::anyhow!("Registry: {e}"))?;

        Ok(())
    }
}

impl Registry {
    /// Generate registries.
    async fn generate_registries(
        version: &Version,
        previous: Option<&Version>,
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        let path =
            path.join(format!("src/generated/v{}.rs", version.to_long_string().replace('.', "_")));

        if tokio::fs::try_exists(&path).await? {
            return Ok(());
        }
        tracing::debug!("Generating registry file \"{}\"", path.display());

        // Tell the macro where the previous version is to inherit from.
        let inheritance = previous
            .map(|p| format!("    inherit = super::{},\n", p.to_long_string().replace('.', "_")))
            .unwrap_or_default();

        // Build the registry macro contents.
        let (current, previous) = Self::get_structures(version, previous, deps).await?;
        let registry = current
            .0
            .iter()
            .map(|(path, current)| (path, current, previous.as_ref().and_then(|p| p.0.get(path))))
            .fold(String::new(), |acc, (_path, _current, _previous)| {
                // TODO: Build Macro

                acc
            });

        // Get the version path and feature.
        let version_path =
            format!("froglight_common::version::V{}", version.to_long_string().replace('.', "_"));
        let feature = format!("\"v{}\"", version.to_long_string().replace('.', "_"));

        tokio::fs::write(
            path,
            format!(
                r"//! This file is generated, do not modify it manually.
//!
//! TODO: Documentation
#![allow(missing_docs)]

froglight_macros::registry_values! {{
    path = crate,
    feature = {feature},
    version = {version_path},
{inheritance}{registry}}}
"
            ),
        )
        .await?;

        Ok(())
    }

    /// Get the structures for the current and previous versions.
    async fn get_structures(
        current: &Version,
        previous: Option<&Version>,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<(Arc<VersionStructures>, Option<Arc<VersionStructures>>)> {
        deps.get_or_retrieve::<DataStructures>().await?;
        deps
            .scoped_fut::<DataStructures, anyhow::Result<(Arc<VersionStructures>, Option<Arc<VersionStructures>>)>>(
                async |data: &mut DataStructures, deps| {
                    let current = data.get_version(current, deps).await?.clone();
                    if let Some(previous) = previous {
                        Ok((current, Some(data.get_version(previous, deps).await?.clone())))
                    } else {
                        Ok((current, None))
                    }

                },
            )
            .await
    }
}
