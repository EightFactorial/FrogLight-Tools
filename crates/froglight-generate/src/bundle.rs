//! A bundle of data

use std::path::Path;

use froglight_definitions::MinecraftVersion;
use serde::{Deserialize, Serialize};

/// A bundle of data that is passed to various data generator modules.
#[derive(Debug, Clone, Hash)]
pub struct GenerateBundle<'a> {
    /// The version to generate data for
    pub version: &'a GenerateVersion,
    /// The root of the project directory
    pub root_dir: &'a Path,
}

impl<'a> GenerateBundle<'a> {
    /// Create a new [`GenerateBundle`].
    #[must_use]
    pub fn new(version: &'a GenerateVersion, root_dir: &'a Path) -> Self {
        Self { version, root_dir }
    }
}

/// A pair of versions to generate data for.
///
/// Using two versions allows for generating data for
/// a version based on another version.
///
/// For example, generating data for `1.20.0` based on `1.20.1`.
#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct GenerateVersion {
    /// The version to generate data for
    pub base: MinecraftVersion,
    /// The jar to use to generate data
    pub jar: MinecraftVersion,
}
