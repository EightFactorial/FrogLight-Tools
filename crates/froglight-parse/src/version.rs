use std::{convert::Infallible, str::FromStr, sync::LazyLock};

use compact_str::CompactString;
use regex::Regex;
use semver::Prerelease;
use serde::{Deserialize, Deserializer, Serialize};

/// A version of Minecraft.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Version {
    /// A release version.
    ///
    /// Examples:
    /// - `1.20`
    /// - `1.20.0`
    /// - `1.20.1`
    Release(semver::Version),
    /// A release candidate version.
    ///
    /// Examples:
    /// - `1.20-rc1`
    /// - `1.20.0-rc1`
    /// - `1.20.1-rc2`
    ReleaseCandidate(semver::Version),
    /// A pre-release version.
    ///
    /// Examples:
    /// - `1.20-pre1`
    /// - `1.20.0-pre1`
    /// - `1.20.1-pre2`
    PreRelease(semver::Version),
    /// A snapshot version.
    ///
    /// Examples:
    /// - `24w40a`
    /// - `24w40b`
    /// - `24w41a`
    Snapshot(semver::Version),
    /// An unknown version.
    Other(CompactString),
}

impl Version {
    /// Create a new [`Version::Release`].
    #[must_use]
    pub const fn new_release(major: u64, minor: u64, patch: u64) -> Self {
        Self::Release(semver::Version::new(major, minor, patch))
    }

    /// Returns `true` if the version is a [`Version::Release`].
    #[must_use]
    pub const fn is_release(&self) -> bool { matches!(self, Self::Release(_)) }

    /// Create a new [`Version::ReleaseCandidate`].
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_rc(major: u64, minor: u64, patch: u64, candidate: u64) -> Self {
        let mut version = semver::Version::new(major, minor, patch);
        version.pre = Prerelease::new(&format!("rc{candidate}")).expect("Invalid pre-release");
        Self::ReleaseCandidate(version)
    }

    /// Returns `true` if the version is a [`Version::ReleaseCandidate`].
    #[must_use]
    pub const fn is_rc(&self) -> bool { matches!(self, Self::ReleaseCandidate(_)) }

    /// Create a new [`Version::PreRelease`].
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_pre(major: u64, minor: u64, patch: u64, prerelease: u64) -> Self {
        let mut version = semver::Version::new(major, minor, patch);
        version.pre = Prerelease::new(&format!("pre{prerelease}")).expect("Invalid pre-release");
        Self::PreRelease(version)
    }

    /// Returns `true` if the version is a [`Version::PreRelease`].
    #[must_use]
    pub const fn is_pre(&self) -> bool { matches!(self, Self::PreRelease(_)) }

    /// Create a new snapshot version.
    ///
    /// # Note
    /// Will fail if the release is not a lowercase ASCII character.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_snapshot(year: u64, week: u64, release: char) -> Option<Self> {
        let release = release.is_ascii_lowercase().then_some(release as u64)?;
        Some(Self::Snapshot(semver::Version::new(year, week, release)))
    }

    /// Returns `true` if the version is a [`Version::Snapshot`].
    #[must_use]
    pub const fn is_snapshot(&self) -> bool { matches!(self, Self::Snapshot(_)) }

    /// Attempt to compare two versions.
    ///
    /// # Note
    /// Will return `None` if the versions are not of the same type,
    /// or if either of the versions are [`Version::Other`].
    #[must_use]
    pub fn compare_relative(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Release(a), Self::Release(b))
            | (Self::ReleaseCandidate(a), Self::ReleaseCandidate(b))
            | (Self::PreRelease(a), Self::PreRelease(b))
            | (Self::Snapshot(a), Self::Snapshot(b)) => a.partial_cmp(b),
            _ => None,
        }
    }

    /// Convert the version to a string, keeping any trailing zeros intact.
    ///
    /// # Examples
    /// ```rust
    /// use froglight_parse::Version;
    ///
    /// let version = Version::new_release(1, 20, 0);
    /// assert_eq!(version.to_string(), "1.20.0");
    /// assert_eq!(version.to_long_string(), "1.20.0");
    /// assert_eq!(version.to_short_string(), "1.20");
    ///
    /// let version = Version::new_rc(1, 20, 0, 1);
    /// assert_eq!(version.to_string(), "1.20.0-rc1");
    /// assert_eq!(version.to_long_string(), "1.20.0-rc1");
    /// assert_eq!(version.to_short_string(), "1.20-rc1");
    ///
    /// let version = Version::new_pre(1, 20, 0, 1);
    /// assert_eq!(version.to_string(), "1.20.0-pre1");
    /// assert_eq!(version.to_long_string(), "1.20.0-pre1");
    /// assert_eq!(version.to_short_string(), "1.20-pre1");
    /// ```
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_long_string(&self) -> CompactString {
        match self {
            Version::Release(version) => {
                format!("{}.{}.{}", version.major, version.minor, version.patch).into()
            }
            Version::ReleaseCandidate(version) | Version::PreRelease(version) => {
                format!("{}.{}.{}-{}", version.major, version.minor, version.patch, version.pre)
                    .into()
            }
            Version::Snapshot(version) => format!(
                "{}w{}{}",
                version.major,
                version.minor,
                char::from(u8::try_from(version.patch).expect("Invalid snapshot release"))
            )
            .into(),
            Version::Other(string) => string.clone(),
        }
    }

    /// Convert the version to a string, removing any trailing zeros.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn to_short_string(&self) -> String {
        match self {
            Version::Release(version) => {
                if version.patch == 0 {
                    format!("{}.{}", version.major, version.minor)
                } else {
                    format!("{}.{}.{}", version.major, version.minor, version.patch)
                }
            }
            Version::ReleaseCandidate(version) | Version::PreRelease(version) => {
                if version.patch == 0 {
                    format!("{}.{}-{}", version.major, version.minor, version.pre)
                } else {
                    format!("{}.{}.{}-{}", version.major, version.minor, version.patch, version.pre)
                }
            }
            Version::Snapshot(version) => format!(
                "{}w{}{}",
                version.major,
                version.minor,
                char::from(u8::try_from(version.patch).expect("Invalid snapshot release"))
            ),
            Version::Other(string) => string.to_string(),
        }
    }
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Release(ver) => write!(f, "Release({ver})"),
            Self::ReleaseCandidate(ver) => write!(f, "ReleaseCandidate({ver})"),
            Self::PreRelease(ver) => write!(f, "PreRelease({ver})"),
            Self::Snapshot(ver) => write!(f, "Snapshot({ver})"),
            Self::Other(ver) => write!(f, "Other({ver})"),
        }
    }
}
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_long_string())
    }
}

impl FromStr for Version {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(parse_release(s)
            .or_else(|| parse_release_candidate(s))
            .or_else(|| parse_pre_release(s))
            .or_else(|| parse_snapshot(s))
            .unwrap_or_else(|| Version::Other(s.into())))
    }
}

impl Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = Version;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string")
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Self::visit_str(self, &value)
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Version::from_str(value).map_err(E::custom)
            }
        }
        deserializer.deserialize_string(Visitor)
    }
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::Release`].
static RELEASE_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(RELEASE_REGEX_STR).unwrap());
/// The string for [`RELEASE_REGEX`].
static RELEASE_REGEX_STR: &str = r"^(\d+)\.(\d+)(\.(\d+))?$";
/// The capture groups for [`RELEASE_REGEX`].
static RELEASE_REGEX_GROUPS: [usize; 3] = [1, 2, 4];

pub(super) fn parse_release(ver: &str) -> Option<Version> {
    let caps = RELEASE_REGEX.captures(ver)?;
    let major = caps.get(RELEASE_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let minor = caps.get(RELEASE_REGEX_GROUPS[1])?.as_str().parse().ok()?;

    let patch = if let Some(cap) = caps.get(RELEASE_REGEX_GROUPS[2]) {
        cap.as_str().parse().ok()?
    } else {
        0
    };

    Some(Version::new_release(major, minor, patch))
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::ReleaseCandidate`].
static RELEASE_CANDIDATE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(RELEASE_CANDIDATE_REGEX_STR).unwrap());
/// The string for [`RELEASE_CANDIDATE_REGEX`].
static RELEASE_CANDIDATE_REGEX_STR: &str = r"^(\d+)\.(\d+)(\.(\d+))?-rc(\d+)$";
/// The capture groups for [`RELEASE_CANDIDATE_REGEX`].
static RELEASE_CANDIDATE_REGEX_GROUPS: [usize; 4] = [1, 2, 4, 5];

pub(super) fn parse_release_candidate(ver: &str) -> Option<Version> {
    let caps = RELEASE_CANDIDATE_REGEX.captures(ver)?;
    let major = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let minor = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[1])?.as_str().parse().ok()?;

    let patch = if let Some(cap) = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[2]) {
        cap.as_str().parse().ok()?
    } else {
        0
    };

    let rc = caps.get(RELEASE_CANDIDATE_REGEX_GROUPS[3])?.as_str().parse().ok()?;
    Some(Version::new_rc(major, minor, patch, rc))
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::PreRelease`].
static PRE_RELEASE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(PRE_RELEASE_REGEX_STR).unwrap());
/// The string for [`PRE_RELEASE_REGEX`].
static PRE_RELEASE_REGEX_STR: &str = r"^(\d+)\.(\d+)(\.(\d+))?-pre(\d+)$";
/// The capture groups for [`PRE_RELEASE_REGEX`].
static PRE_RELEASE_REGEX_GROUPS: [usize; 4] = [1, 2, 4, 5];

pub(super) fn parse_pre_release(ver: &str) -> Option<Version> {
    let caps = PRE_RELEASE_REGEX.captures(ver)?;
    let major = caps.get(PRE_RELEASE_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let minor = caps.get(PRE_RELEASE_REGEX_GROUPS[1])?.as_str().parse().ok()?;

    let patch = if let Some(cap) = caps.get(PRE_RELEASE_REGEX_GROUPS[2]) {
        cap.as_str().parse().ok()?
    } else {
        0
    };

    let pre = caps.get(PRE_RELEASE_REGEX_GROUPS[3])?.as_str().parse().ok()?;
    Some(Version::new_pre(major, minor, patch, pre))
}

// -----------------------------------------------------------------------------

/// The [`Regex`] for [`MinecraftVersion::Snapshot`].
static SNAPSHOT_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(SNAPSHOT_REGEX_STR).unwrap());
/// The string for [`SNAPSHOT_REGEX`].
static SNAPSHOT_REGEX_STR: &str = r"^(\d\d)w(\d\d)([a-z])$";
/// The capture groups for [`SNAPSHOT_REGEX`].
static SNAPSHOT_REGEX_GROUPS: [usize; 3] = [1, 2, 3];

pub(super) fn parse_snapshot(ver: &str) -> Option<Version> {
    let caps = SNAPSHOT_REGEX.captures(ver)?;
    let year = caps.get(SNAPSHOT_REGEX_GROUPS[0])?.as_str().parse().ok()?;
    let week = caps.get(SNAPSHOT_REGEX_GROUPS[1])?.as_str().parse().ok()?;
    let release = caps.get(SNAPSHOT_REGEX_GROUPS[2])?.as_str().parse().ok()?;
    Version::new_snapshot(year, week, release)
}
