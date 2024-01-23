use regex::Regex;
use serde::{Deserialize, Deserializer};

use crate::VersionManifest;

/// A representation of a version number.
///
/// This is incomplete, and only contains the data we need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Version {
    /// A release version, such as 1.20.0
    Release(semver::Version),
    /// A release candidate, such as 1.20.0-rc1
    ReleaseCandidate(semver::Version),
    /// A prerelease version, such as 1.20.0-pre1
    PreRelease(semver::Version),
    /// A snapshot version, such as 24w03b
    Snapshot(String),
    /// Any version that doesn't fit into the above categories.
    Other(String),
}

impl Version {
    /// Create a new [`Version::Release`]
    #[must_use]
    pub const fn new_rel(major: u64, minor: u64, patch: u64) -> Self {
        Self::Release(semver::Version::new(major, minor, patch))
    }

    /// Create a new [`Version::ReleaseCandidate`]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_rc(major: u64, minor: u64, patch: u64, rc: u64) -> Self {
        let mut sversion = semver::Version::new(major, minor, patch);
        sversion.pre = semver::Prerelease::new(format!("rc{}", rc).as_str()).unwrap();
        Self::ReleaseCandidate(sversion)
    }

    /// Create a new [`Version::PreRelease`]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_pre(major: u64, minor: u64, patch: u64, pre: u64) -> Self {
        let mut sversion = semver::Version::new(major, minor, patch);
        sversion.pre = semver::Prerelease::new(format!("pre{}", pre).as_str()).unwrap();
        Self::PreRelease(sversion)
    }

    /// Create a new [`Version::Snapshot`]
    #[must_use]
    pub fn new_snapshot(s: impl AsRef<str>) -> Self { Self::Snapshot(s.as_ref().into()) }

    /// Create a new [`Version::Other`]
    #[must_use]
    pub fn new_other(s: impl AsRef<str>) -> Self { Self::Other(s.as_ref().into()) }

    /// Try to create a new [`Version`] from a string.
    #[must_use]
    pub fn try_from_string(version: &str) -> Option<Self> {
        Self::try_from_semver(&semver::Version::parse(version).ok()?)
    }

    /// Try to create a new [`Version`] from a [`semver::Version`].
    #[must_use]
    pub fn try_from_semver(version: &semver::Version) -> Option<Self> {
        if version.pre.starts_with("rc") {
            Some(Self::ReleaseCandidate(version.clone()))
        } else if version.pre.starts_with("pre") {
            Some(Self::PreRelease(version.clone()))
        } else if version.pre.is_empty() {
            Some(Self::Release(version.clone()))
        } else {
            None
        }
    }

    /// This method is used to determine if a version is less than an another
    /// version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is less than `other`.
    #[must_use]
    pub fn lt_man(&self, _other: &Self, _manifest: &VersionManifest) -> bool { todo!() }

    /// This method is used to determine if a version is less than or equal to
    /// another version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is less than or equal to `other`.
    #[must_use]
    pub fn le_man(&self, _other: &Self, _manifest: &VersionManifest) -> bool { todo!() }

    /// This method is used to determine if a version is greater than an another
    /// version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is greater than `other`.
    #[must_use]
    pub fn gt_man(&self, _other: &Self, _manifest: &VersionManifest) -> bool { todo!() }

    /// This method is used to determine if a version is greater than or equal
    /// to another version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is greater than or equal to `other`.
    #[must_use]
    pub fn ge_man(&self, _other: &Self, _manifest: &VersionManifest) -> bool { todo!() }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;

        if let Ok(sversion) = semver::Version::parse(&string) {
            // This should catch most versions
            // Does not work on version without a patch, such as `1.20`
            if let Some(version) = Self::try_from_semver(&sversion) {
                Ok(version)
            } else {
                Ok(Self::Other(sversion.to_string()))
            }
        } else if string.split('.').count() == 2 {
            // This should catch `1.20`, `1.20-rc1`, etc

            let mut index = string.len();
            if let Some(pos) = string.find('-') {
                index = pos;
            }

            // Split the string into the version and the pre information
            let (version, pre) = string.split_at(index);

            // Parse the version
            let mut split = version.split('.');
            let major = split.next().unwrap().parse::<u64>().unwrap();
            let minor = split.next().unwrap().parse::<u64>().unwrap();

            // Determine release type
            if let Some(pre) = pre.strip_prefix("-rc") {
                Ok(Self::new_rc(major, minor, 0, pre.parse().unwrap()))
            } else if let Some(pre) = pre.strip_prefix("-pre") {
                Ok(Self::new_pre(major, minor, 0, pre.parse().unwrap()))
            } else {
                Ok(Self::new_rel(major, minor, 0))
            }
        } else {
            // This should catch `20w45a`, `20w45b`, etc
            let re = Regex::new(r"(\d+)(w)(\d+)([a-z])").unwrap();
            if let Some(cap) = re.captures(&string).and_then(|c| c.get(0)) {
                Ok(Self::new_snapshot(cap.as_str()))
            } else {
                Ok(Self::Other(string))
            }
        }
    }
}

#[test]
fn version_deserialization() {
    #[derive(Deserialize)]
    struct VersionTest {
        ver: Version,
    }

    // Explicit versions
    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20.0"}"#).unwrap();
    assert_eq!(test.ver, Version::new_rel(1, 20, 0));

    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20.0-rc1"}"#).unwrap();
    assert_eq!(test.ver, Version::new_rc(1, 20, 0, 1));

    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20.0-pre1"}"#).unwrap();
    assert_eq!(test.ver, Version::new_pre(1, 20, 0, 1));

    // Implicit versions
    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20"}"#).unwrap();
    assert_eq!(test.ver, Version::new_rel(1, 20, 0));

    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20-rc2"}"#).unwrap();
    assert_eq!(test.ver, Version::new_rc(1, 20, 0, 2));

    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20-pre2"}"#).unwrap();
    assert_eq!(test.ver, Version::new_pre(1, 20, 0, 2));

    // Snapshots
    let test: VersionTest = serde_json::from_str(r#"{"ver":"20w45a"}"#).unwrap();
    assert_eq!(test.ver, Version::new_snapshot("20w45a"));

    let test: VersionTest = serde_json::from_str(r#"{"ver":"20w45b"}"#).unwrap();
    assert_eq!(test.ver, Version::new_snapshot("20w45b"));

    // Other
    let test: VersionTest = serde_json::from_str(r#"{"ver":"1.20.0-other"}"#).unwrap();
    assert_eq!(test.ver, Version::new_other("1.20.0-other"));

    let test: VersionTest = serde_json::from_str(r#"{"ver":"random-text"}"#).unwrap();
    assert_eq!(test.ver, Version::new_other("random-text"));
}
