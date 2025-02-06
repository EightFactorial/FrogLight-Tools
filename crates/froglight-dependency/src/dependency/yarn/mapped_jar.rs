//! TODO

use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;

use crate::{
    container::DependencyContainer,
    dependency::{
        minecraft::MinecraftJar,
        yarn::{TinyRemapper, YarnMappings},
    },
    version::Version,
};

/// Mapped Client and Server JAR paths.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct MappedJar {
    client: HashMap<Version, PathBuf>,
    server: HashMap<Version, PathBuf>,
}

impl MappedJar {
    /// Get the [`Path`] of the mapped client jar for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn client(&self, version: &Version) -> Option<&Path> {
        self.client.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the mapped server jar for the given version.
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
            deps.get_or_retrieve::<MinecraftJar>().await?;
            deps.scoped_fut::<MinecraftJar, anyhow::Result<()>>(
                async |jars: &mut MinecraftJar, deps: &mut DependencyContainer| {
                    let client = jars.get_client(version, deps).await?;
                    self.client
                        .insert(version.clone(), Self::map_jar(version, client, deps).await?);
                    Ok(())
                },
            )
            .await
            .map_err(|err| anyhow::anyhow!("MappedJar: {err}"))?;
        }

        Ok(self.client(version).unwrap())
    }

    /// Get the [`Path`] of the mapped server jar for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn server(&self, version: &Version) -> Option<&Path> {
        self.server.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the mapped server jar for the given version.
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
            deps.get_or_retrieve::<MinecraftJar>().await?;
            deps.scoped_fut::<MinecraftJar, anyhow::Result<()>>(
                async |jars: &mut MinecraftJar, deps: &mut DependencyContainer| {
                    let server = jars.get_server(version, deps).await?;
                    self.server
                        .insert(version.clone(), Self::map_jar(version, server, deps).await?);
                    Ok(())
                },
            )
            .await
            .map_err(|err| anyhow::anyhow!("MappedJar: {err}"))?;
        }

        Ok(self.server(version).unwrap())
    }
}

impl MappedJar {
    async fn map_jar(
        version: &Version,
        jar: &Path,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<PathBuf> {
        let out = jar.with_file_name(format!(
            "{}-mapped.jar",
            jar.file_name().unwrap().to_string_lossy().split_once('.').unwrap().0
        ));

        if tokio::fs::try_exists(&out).await? {
            tracing::debug!("Using \"{}\"", out.display());
        } else {
            // Retrieve the mappings and map the jar
            deps.get_or_retrieve::<YarnMappings>().await?;
            deps.scoped_fut::<YarnMappings, anyhow::Result<()>>(
                async |mappings: &mut YarnMappings, deps: &mut DependencyContainer| {
                    let mappings = mappings.get_version(version, deps).await?;
                    deps.get_or_retrieve::<TinyRemapper>()
                        .await?
                        .remap_jar(jar, &out, mappings)
                        .await
                },
            )
            .await?;
        }

        Ok(out)
    }
}
