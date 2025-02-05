use std::{io::Read, path::PathBuf};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use zip::ZipArchive;

use super::YarnMaven;
use crate::{container::DependencyContainer, version::Version};

/// A collection of [`YarnMapping`]s.
#[derive(Debug, Default, Clone, PartialEq, Eq, Dependency)]
#[dep(path = crate)]
pub struct YarnMappings(HashMap<Version, YarnMapping>);

/// Mappings for a specific [`Version`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YarnMapping(PathBuf);

impl YarnMappings {
    const JAR_FILENAME: &str = "yarn-mergedv2.jar";
    const MAPPINGS_FILENAME: &str = "mappings.tiny";

    /// Returns the [`YarnMapping`] for the given [`Version`].
    ///
    /// Returns `None` if the [`YarnMapping`] is not found.
    #[must_use]
    pub fn version(&self, version: &Version) -> Option<&YarnMapping> { self.0.get(version) }

    /// Returns the [`YarnMapping`] for the given [`Version`].
    ///
    /// # Errors
    /// Returns an error if there was an error retrieving the [`YarnMapping`].
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&YarnMapping> {
        if !self.0.contains_key(version) {
            let mappings_path =
                deps.cache.join(version.to_long_string()).join(Self::MAPPINGS_FILENAME);

            if tokio::fs::try_exists(&mappings_path).await? {
                tracing::debug!("Reading \"{}\"", mappings_path.display());
            } else {
                let jar_path = deps.cache.join(version.to_long_string()).join(Self::JAR_FILENAME);

                if tokio::fs::try_exists(&jar_path).await? {
                    tracing::debug!("Reading \"{}\"", jar_path.display());
                } else {
                    let url =
                        deps.get_or_retrieve::<YarnMaven>().await?.get_url(version).ok_or_else(
                            || anyhow::anyhow!("Failed to get YarnMaven URL for version {version}"),
                        )?;
                    tracing::debug!("Retrieving \"{url}\"");

                    let response = deps.client.get(url).send().await?.bytes().await?;
                    tokio::fs::write(&jar_path, response).await?;
                }

                let mut zip = ZipArchive::new(std::io::Cursor::new(std::fs::read(jar_path)?))?;
                let mut mappings_buffer = Vec::new();

                zip.by_name("mappings/mappings.tiny")?.read_to_end(&mut mappings_buffer)?;
                tokio::fs::write(&mappings_path, mappings_buffer).await?;
            }

            self.0.insert(version.clone(), YarnMapping(mappings_path));
        }

        Ok(self.0.get(version).unwrap())
    }
}

impl std::ops::Deref for YarnMapping {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl std::ops::DerefMut for YarnMapping {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
