//! TODO

use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;

use super::MappedJar;
use crate::{container::DependencyContainer, dependency::vineflower::Vineflower, version::Version};

/// Decompiled Client and Server JAR paths.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct DecompiledJar {
    client: HashMap<Version, PathBuf>,
    server: HashMap<Version, PathBuf>,
}

impl DecompiledJar {
    /// Get the [`Path`] of the decompiled client jar for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn client(&self, version: &Version) -> Option<&Path> {
        self.client.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the decompiled server jar for the given version.
    ///
    /// # Errors
    /// Returns an error if there was an error getting the path.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_client(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&Path> {
        if !self.client.contains_key(version) {
            deps.get_or_retrieve::<MappedJar>().await?;
            deps.scoped_fut::<MappedJar, anyhow::Result<()>>(
                async |jars: &mut MappedJar, deps: &mut DependencyContainer| {
                    let client = jars.get_client(version, deps).await?;
                    self.client.insert(version.clone(), Self::decompile_jar(client, deps).await?);
                    Ok(())
                },
            )
            .await?;
        }

        Ok(self.client(version).unwrap())
    }

    /// Get the [`Path`] of the decompiled server jar for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn server(&self, version: &Version) -> Option<&Path> {
        self.server.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the decompiled server jar for the given version.
    ///
    /// # Errors
    /// Returns an error if there was an error getting the path.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_server(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&Path> {
        if !self.server.contains_key(version) {
            deps.get_or_retrieve::<MappedJar>().await?;
            deps.scoped_fut::<MappedJar, anyhow::Result<()>>(
                async |jars: &mut MappedJar, deps: &mut DependencyContainer| {
                    let server = jars.get_server(version, deps).await?;
                    self.server.insert(version.clone(), Self::decompile_jar(server, deps).await?);
                    Ok(())
                },
            )
            .await?;
        }

        Ok(self.server(version).unwrap())
    }
}

impl DecompiledJar {
    async fn decompile_jar(jar: &Path, deps: &mut DependencyContainer) -> anyhow::Result<PathBuf> {
        let out = jar.with_file_name(format!(
            "{}-decompiled",
            jar.file_name().unwrap().to_string_lossy().split_once('.').unwrap().0
        ));

        if tokio::fs::try_exists(&out).await? {
            tracing::debug!("Using \"{}\"", out.display());
        } else {
            // Retrieve the decompiler and decompile the jar
            deps.get_or_retrieve::<Vineflower>().await?.decompile_jar(jar, &out).await?;
        }

        Ok(out)
    }
}
