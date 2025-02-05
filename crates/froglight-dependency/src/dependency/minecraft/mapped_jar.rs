//! TODO

use std::path::{Path, PathBuf};

use froglight_tool_macros::Dependency;
use hashbrown::HashMap;

use crate::{container::DependencyContainer, version::Version};

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
    pub fn get_client(
        &mut self,
        version: Version,
        _deps: &mut DependencyContainer,
    ) -> anyhow::Result<&Path> {
        self.client(&version).map_or_else(|| todo!(), Ok)
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
    pub fn get_server(
        &mut self,
        version: Version,
        _deps: &mut DependencyContainer,
    ) -> anyhow::Result<&Path> {
        self.server(&version).map_or_else(|| todo!(), Ok)
    }
}

impl MappedJar {}
