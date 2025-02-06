//! TODO

use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;

use crate::{
    container::DependencyContainer,
    dependency::mojang::{release_manifest::ReleaseDownload, ReleaseManifests},
    version::Version,
};

/// Client and Server JAR paths.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct MinecraftJar {
    client: HashMap<Version, PathBuf>,
    server: HashMap<Version, PathBuf>,
}

impl MinecraftJar {
    /// Get the [`Path`] of the client jar for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn client(&self, version: &Version) -> Option<&Path> {
        self.client.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the server jar for the given version.
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
            deps.get_or_retrieve::<ReleaseManifests>().await?;
            deps.scoped_fut::<ReleaseManifests, anyhow::Result<()>>(
                async |manifest: &mut ReleaseManifests, deps| {
                    MinecraftJar::download_and_cache(
                        version,
                        &manifest.get_release(version, deps).await?.downloads.client,
                        &mut self.client,
                        deps,
                    )
                    .await
                },
            )
            .await
            .map_err(|err| anyhow::anyhow!("MinecraftJar: {err}"))?;
        }

        Ok(self.client(version).unwrap())
    }

    /// Get the [`Path`] of the server jar for the given version.
    ///
    /// Returns `None` if the path is not yet known.
    #[must_use]
    pub fn server(&self, version: &Version) -> Option<&Path> {
        self.server.get(version).map(PathBuf::as_path)
    }

    /// Get the [`Path`] of the server jar for the given version.
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
            deps.get_or_retrieve::<ReleaseManifests>().await?;
            deps.scoped_fut::<ReleaseManifests, anyhow::Result<()>>(
                async |manifest: &mut ReleaseManifests, deps| {
                    MinecraftJar::download_and_cache(
                        version,
                        &manifest.get_release(version, deps).await?.downloads.server,
                        &mut self.server,
                        deps,
                    )
                    .await
                },
            )
            .await?;
        }

        Ok(self.server(version).unwrap())
    }
}

impl MinecraftJar {
    async fn download_and_cache(
        version: &Version,
        download: &ReleaseDownload,
        storage: &mut HashMap<Version, PathBuf>,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<()> {
        // Create the version cache directory if it doesn't exist
        let mut cache = deps.cache.join(version.to_long_string());
        if !tokio::fs::try_exists(&cache).await? {
            tokio::fs::create_dir_all(&cache).await?;
        }

        // Get the path to the jar
        cache.push(download.url.split('/').next_back().unwrap());
        if tokio::fs::try_exists(&cache).await? {
            tracing::debug!("Using \"{}\"", cache.display());
        } else {
            tracing::debug!("Retrieving \"{}\"", download.url);

            // Download the jar if it doesn't exist
            let response = deps.client.get(&download.url).send().await?;
            let bytes = response.bytes().await?;
            tokio::fs::write(&cache, &bytes).await?;
        }
        storage.insert(version.clone(), cache);

        Ok(())
    }
}
