use std::fmt::Display;

use regex::Regex;
use serde::{Deserialize, Deserializer};

use crate::VersionManifest;

/// A representation of a version number.
///
/// Version numbers can't be compared directly, as snapshots
/// follow a different naming scheme than release versions.
///
/// Instead, a [`VersionManifest`] can be used with [`Version::lt_man`],
/// [`Version::le_man`], [`Version::gt_man`], and [`Version::ge_man`] to compare
/// release order.
///
/// # Examples
/// ```rust
/// use froglight_data::Version;
///
/// let version = Version::new_rel(1, 20, 0);
/// assert_eq!(version.to_string(), "1.20.0");
/// assert_eq!(version.to_short_string(), "1.20");
///
/// let version = Version::new_rel(1, 20, 1);
/// assert_eq!(version.to_string(), "1.20.1");
///
/// let version = Version::new_rc(1, 20, 1, 1);
/// assert_eq!(version.to_string(), "1.20.1-rc1");
///
/// let version = Version::new_pre(1, 20, 2, 3);
/// assert_eq!(version.to_string(), "1.20.2-pre3");
///
/// let version = Version::new_snapshot("20w45b");
/// assert_eq!(version.to_string(), "20w45b");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
        sversion.pre = semver::Prerelease::new(format!("rc{rc}").as_str()).unwrap();
        Self::ReleaseCandidate(sversion)
    }

    /// Create a new [`Version::PreRelease`]
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_pre(major: u64, minor: u64, patch: u64, pre: u64) -> Self {
        let mut sversion = semver::Version::new(major, minor, patch);
        sversion.pre = semver::Prerelease::new(format!("pre{pre}").as_str()).unwrap();
        Self::PreRelease(sversion)
    }

    /// Create a new [`Version::Snapshot`]
    #[must_use]
    pub fn new_snapshot(s: impl AsRef<str>) -> Self { Self::Snapshot(s.as_ref().into()) }

    /// Create a new [`Version::Other`]
    #[must_use]
    pub fn new_other(s: impl AsRef<str>) -> Self { Self::Other(s.as_ref().into()) }

    /// Try to create a new [`Version`] from a string.
    ///
    /// # Examples
    /// ```rust
    /// use froglight_data::Version;
    ///
    /// let version = Version::from_string("1.20.0").unwrap();
    /// assert_eq!(version, Version::new_rel(1, 20, 0));
    ///
    /// let version = Version::from_string("1.20.0-rc1").unwrap();
    /// assert_eq!(version, Version::new_rc(1, 20, 0, 1));
    /// ```
    ///
    /// # Panics
    /// - Panics if the string is not a valid version.
    #[must_use]
    pub fn from_string(string: &str) -> Option<Self> {
        if let Ok(sversion) = semver::Version::parse(string) {
            // This should catch most versions
            // Does not work on version without a patch, such as `1.20`
            if let Some(version) = Self::try_from_semver(&sversion) {
                Some(version)
            } else {
                Some(Self::Other(sversion.to_string()))
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
            let major = split.next().unwrap().parse::<u64>();
            let minor = split.next().unwrap().parse::<u64>();

            let (Ok(major), Ok(minor)) = (major, minor) else {
                return Some(Self::Other(string.to_owned()));
            };

            // Determine release type
            if let Some(pre) = pre.strip_prefix("-rc") {
                Some(Self::new_rc(major, minor, 0, pre.parse().unwrap()))
            } else if let Some(pre) = pre.strip_prefix("-pre") {
                Some(Self::new_pre(major, minor, 0, pre.parse().unwrap()))
            } else {
                Some(Self::new_rel(major, minor, 0))
            }
        } else {
            // This should catch `20w45a`, `20w45b`, etc
            let re = Regex::new(r"\d+w\d+[a-z]").unwrap();
            if re.is_match(string) {
                Some(Self::Snapshot(string.to_owned()))
            } else {
                Some(Self::Other(string.to_owned()))
            }
        }
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

    /// Convert the [`Version`] to a [`String`].
    ///
    /// This will remove the patch version if it is 0.
    ///
    /// # Examples
    /// ```rust
    /// use froglight_data::Version;
    ///
    /// let version = Version::new_rel(1, 20, 0);
    /// assert_eq!(version.to_string(), "1.20.0");
    /// assert_eq!(version.to_short_string(), "1.20");
    ///
    /// let version = Version::new_rel(1, 20, 1);
    /// assert_eq!(version.to_string(), "1.20.1");
    /// assert_eq!(version.to_short_string(), "1.20.1");
    ///
    /// let version = Version::new_rc(1, 20, 0, 1);
    /// assert_eq!(version.to_string(), "1.20.0-rc1");
    /// assert_eq!(version.to_short_string(), "1.20-rc1");
    ///
    /// let version = Version::new_pre(1, 20, 0, 1);
    /// assert_eq!(version.to_string(), "1.20.0-pre1");
    /// assert_eq!(version.to_short_string(), "1.20-pre1");
    ///
    /// let version = Version::new_snapshot("20w45b");
    /// assert_eq!(version.to_string(), "20w45b");
    /// ```
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_short_string(&self) -> String {
        match self {
            Self::Release(version)
            | Self::PreRelease(version)
            | Self::ReleaseCandidate(version) => {
                let re = Regex::new(r"\d+\.\d+\.0.*").unwrap();
                if re.is_match(&version.to_string()) {
                    if version.pre.is_empty() {
                        format!("{}.{}", version.major, version.minor)
                    } else {
                        format!("{}.{}-{}", version.major, version.minor, version.pre)
                    }
                } else {
                    version.to_string()
                }
            }
            Self::Snapshot(version) | Self::Other(version) => version.clone(),
        }
    }

    /// This method is used to determine if a version is less than another.
    ///
    /// Requires both versions to be of the same type.
    ///
    /// Only works for [`Version::Release`], [`Version::ReleaseCandidate`], and
    /// [`Version::PreRelease`].
    #[must_use]
    pub fn try_lt(&self, other: &Self) -> Option<bool> {
        match (self, other) {
            (Version::Release(a), Version::Release(b))
            | (Version::ReleaseCandidate(a), Version::ReleaseCandidate(b))
            | (Version::PreRelease(a), Version::PreRelease(b)) => Some(a < b),
            _ => None,
        }
    }

    /// This method is used to determine if a version is less than or equal to
    /// another.
    ///
    /// Requires both versions to be of the same type.
    ///
    /// Only works for [`Version::Release`], [`Version::ReleaseCandidate`], and
    /// [`Version::PreRelease`].
    #[must_use]
    pub fn try_le(&self, other: &Self) -> Option<bool> {
        match (self, other) {
            (Version::Release(a), Version::Release(b))
            | (Version::ReleaseCandidate(a), Version::ReleaseCandidate(b))
            | (Version::PreRelease(a), Version::PreRelease(b)) => Some(a <= b),
            _ => None,
        }
    }

    /// This method is used to determine if a version is greater than another.
    ///
    /// Requires both versions to be of the same type.
    ///
    /// Only works for [`Version::Release`], [`Version::ReleaseCandidate`], and
    /// [`Version::PreRelease`].
    #[must_use]
    pub fn try_gt(&self, other: &Self) -> Option<bool> {
        match (self, other) {
            (Version::Release(a), Version::Release(b))
            | (Version::ReleaseCandidate(a), Version::ReleaseCandidate(b))
            | (Version::PreRelease(a), Version::PreRelease(b)) => Some(a > b),
            _ => None,
        }
    }

    /// This method is used to determine if a version is greater than or equal
    /// to another.
    ///
    /// Requires both versions to be of the same type.
    ///
    /// Only works for [`Version::Release`], [`Version::ReleaseCandidate`], and
    /// [`Version::PreRelease`].
    #[must_use]
    pub fn try_ge(&self, other: &Self) -> Option<bool> {
        match (self, other) {
            (Version::Release(a), Version::Release(b))
            | (Version::ReleaseCandidate(a), Version::ReleaseCandidate(b))
            | (Version::PreRelease(a), Version::PreRelease(b)) => Some(a >= b),
            _ => None,
        }
    }

    /// This method is used to determine if a version is less than an another
    /// version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is less than `other`.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn lt_man(&self, other: &Self, manifest: &VersionManifest) -> bool {
        let self_index = manifest.versions.iter().position(|v| &v.id == self).unwrap();
        let other_index = manifest.versions.iter().position(|v| &v.id == other).unwrap();
        self_index < other_index
    }

    /// This method is used to determine if a version is less than or equal to
    /// another version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is less than or equal to `other`.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn le_man(&self, other: &Self, manifest: &VersionManifest) -> bool {
        let self_index = manifest.versions.iter().position(|v| &v.id == self).unwrap();
        let other_index = manifest.versions.iter().position(|v| &v.id == other).unwrap();
        self_index <= other_index
    }

    /// This method is used to determine if a version is greater than an another
    /// version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is greater than `other`.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn gt_man(&self, other: &Self, manifest: &VersionManifest) -> bool {
        let self_index = manifest.versions.iter().position(|v| &v.id == self).unwrap();
        let other_index = manifest.versions.iter().position(|v| &v.id == other).unwrap();
        self_index > other_index
    }

    /// This method is used to determine if a version is greater than or equal
    /// to another version.
    ///
    /// Requires a [`VersionManifest`] to determine the order of versions.
    ///
    /// Returns `true` if `self` is greater than or equal to `other`.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn ge_man(&self, other: &Self, manifest: &VersionManifest) -> bool {
        let self_index = manifest.versions.iter().position(|v| &v.id == self).unwrap();
        let other_index = manifest.versions.iter().position(|v| &v.id == other).unwrap();
        self_index >= other_index
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::from_string(&string).ok_or_else(|| serde::de::Error::custom("Invalid version"))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Release(version)
            | Self::PreRelease(version)
            | Self::ReleaseCandidate(version) => version.fmt(f),
            Self::Snapshot(version) | Self::Other(version) => version.fmt(f),
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
