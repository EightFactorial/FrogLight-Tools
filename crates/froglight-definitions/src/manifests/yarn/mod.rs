use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::MinecraftVersion;

#[cfg(test)]
mod tests;

/// A manifest for Yarn mappings.
///
/// Used to identify the latest yarn mappings build
/// for a given Minecraft version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "metadata")]
pub struct YarnManifest {
    /// Version information for the Yarn mappings.
    #[serde(rename = "versioning")]
    pub versions: YarnVersions,
}
impl YarnManifest {
    /// The URL to the [`YarnManifest`].
    pub const URL: &'static str = "https://maven.fabricmc.net/net/fabricmc/yarn/maven-metadata.xml";

    /// Get all Yarn versions for a specific [`MinecraftVersion`].
    #[must_use]
    pub fn get_versions(&self, version: &MinecraftVersion) -> Vec<&YarnVersion> {
        let version_str = version.to_short_string();
        let mut version_list = Vec::new();

        for version in &self.versions.versions.versions {
            if version.0.starts_with(&version_str)
                && version.0.get(version_str.len()..).is_some_and(|s| s.starts_with(['+', '.']))
            {
                version_list.push(version);
            }
        }

        version_list
    }

    /// Get the latest [`YarnVersion`] for a specific [`MinecraftVersion`].
    ///
    /// Returns `None` if no versions are found.
    #[must_use]
    pub fn get_latest(&self, version: &MinecraftVersion) -> Option<&YarnVersion> {
        self.get_versions(version).into_iter().max_by(|&a, &b| a.split().1.cmp(&b.split().1))
    }
}

/// Version information for the Yarn mappings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YarnVersions {
    /// The latest version of Yarn.
    pub latest: YarnVersion,
    /// The latest release of Yarn.
    pub release: YarnVersion,
    /// All Yarn versions.
    pub versions: YarnVersionList,
}

/// A list of all Yarn versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YarnVersionList {
    /// A list of Yarn versions.
    #[serde(rename = "version")]
    pub versions: Vec<YarnVersion>,
}

/// A Yarn version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct YarnVersion(#[serde(rename = "$text")] pub CompactString);

impl YarnVersion {
    /// Split the Yarn version into a [`MinecraftVersion`] and a build number.
    ///
    /// All yarn versions are stored as strings.
    ///
    /// The first part is the Minecraft version,
    /// and the second part is the build number.
    ///
    /// Strings are either formatted as `19w13a.1`
    /// or `1.21-pre4+build.2`
    ///
    /// # Panics
    /// Panics if the version string is not formatted correctly.
    #[must_use]
    pub fn split(&self) -> (MinecraftVersion, u32) {
        // Get the build number from the end of the string
        let split_last = self.0.split('.').last().expect("YarnVersion split failed");
        let build = split_last.parse().expect("YarnVersion build parse failed");

        // Trim up to "+build.X" from the end of the string
        let version = self
            .0
            .as_str()
            .trim_end_matches(split_last)
            .trim_end_matches('.')
            .trim_end_matches("+build");

        // Infallible parse
        (version.parse().unwrap(), build)
    }
}

impl AsRef<str> for YarnVersion {
    fn as_ref(&self) -> &str { self.0.as_str() }
}
impl AsRef<CompactString> for YarnVersion {
    fn as_ref(&self) -> &CompactString { &self.0 }
}

impl std::fmt::Display for YarnVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
