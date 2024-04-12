use std::{cmp::Ordering, convert::Infallible, str::FromStr};

use compact_str::CompactString;
use semver::{Error as SemverError, Prerelease, Version};
use serde::{Deserialize, Serialize};

pub(crate) mod regex;

#[cfg(test)]
mod tests;

/// A version of Minecraft.
///
/// ---
///
/// ### Warning
/// Very old versions will not be parsed correctly!
///
/// For example, `Beta 1.8 Pre-release 1` will be parsed as
/// `Other("Beta 1.8 Pre-release 1")`.
#[derive(Clone, Hash)]
pub enum MinecraftVersion {
    /// A `Release` version of Minecraft.
    ///
    /// For example, `1.20.0`.
    Release(Version),

    /// A `Release Candidate` version of Minecraft.
    ///
    /// For example, `1.20.0-rc2`.
    ReleaseCandidate(Version),

    /// A `PreRelease` version of Minecraft.
    ///
    /// For example, `1.20.0-pre3`.
    PreRelease(Version),

    /// A `Snapshot` version of Minecraft.
    ///
    /// For example, `24w13a`.
    Snapshot(Version),

    /// Other versions of Minecraft.
    Other(CompactString),
}

impl MinecraftVersion {
    /// Creates a new [`MinecraftVersion::Release`].
    #[must_use]
    pub const fn new_release(major: u64, minor: u64, patch: u64) -> Self {
        Self::Release(Version::new(major, minor, patch))
    }

    /// Creates a new [`MinecraftVersion::ReleaseCandidate`].
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn new_release_candidate(
        major: u64,
        minor: u64,
        patch: u64,
        rc: u64,
    ) -> Result<Self, SemverError> {
        Ok(Self::ReleaseCandidate(Version {
            pre: Prerelease::new(&rc.to_string())?,
            ..Version::new(major, minor, patch)
        }))
    }

    /// Creates a new [`MinecraftVersion::PreRelease`].
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn new_pre_release(
        major: u64,
        minor: u64,
        patch: u64,
        pre: u64,
    ) -> Result<Self, SemverError> {
        Ok(Self::PreRelease(Version {
            pre: Prerelease::new(&pre.to_string())?,
            ..Version::new(major, minor, patch)
        }))
    }

    /// Creates a new [`MinecraftVersion::Snapshot`].
    #[allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]
    pub fn new_snapshot(year: u64, week: u64, patch: char) -> Result<Self, SemverError> {
        Ok(Self::Snapshot(Version {
            pre: Prerelease::new(&patch.to_string())?,
            ..Version::new(year, week, 0)
        }))
    }

    /// Creates a new [`MinecraftVersion::Other`].
    #[must_use]
    pub fn new_other(value: &(impl AsRef<str> + ?Sized)) -> Self {
        Self::Other(CompactString::new(value))
    }

    /// Returns `true` if the two [`MinecraftVersions`](MinecraftVersion) are
    /// the same.
    ///
    /// # Examples
    /// ```rust
    /// use froglight_definitions::MinecraftVersion;
    ///
    /// let release_1 = MinecraftVersion::new_release(1, 20, 0);
    /// assert!(release_1.is_same(&release_1));
    ///
    /// let release_2 = MinecraftVersion::new_release(1, 20, 4);
    /// assert!(release_2.is_same(&release_2));
    ///
    /// // The two versions are not the same.
    /// assert!(!release_1.is_same(&release_2));
    ///
    /// let snapshot = MinecraftVersion::new_snapshot(24, 3, 'b').unwrap();
    /// assert!(snapshot.is_same(&snapshot));
    ///
    /// // A snapshot and release will never be the same.
    /// assert!(!snapshot.is_same(&release_1));
    /// ```
    #[must_use]
    pub fn is_same(&self, other: &Self) -> bool { self.try_compare(other) == Some(Ordering::Equal) }

    /// Attempts to compare two [`MinecraftVersions`](MinecraftVersion).
    ///
    /// If both versions are not of the same variant, `None` is returned.
    ///
    /// # Examples
    /// ```rust
    /// use froglight_definitions::MinecraftVersion;
    ///
    /// let release_1 = MinecraftVersion::new_release(1, 20, 0);
    /// let release_2 = MinecraftVersion::new_release(1, 20, 4);
    ///
    /// // `1.20.0` is less than `1.20.4`
    /// assert_eq!(release_1.try_compare(&release_2), Some(std::cmp::Ordering::Less));
    ///
    /// // `1.20.0` is equal to `1.20.0`
    /// assert_eq!(release_1.try_compare(&release_1), Some(std::cmp::Ordering::Equal));
    ///
    /// // `1.20.4` is greater than `1.20.0`
    /// assert_eq!(release_2.try_compare(&release_1), Some(std::cmp::Ordering::Greater));
    ///
    /// let snapshot_1 = MinecraftVersion::new_snapshot(24, 3, 'b').unwrap();
    /// let snapshot_2 = MinecraftVersion::new_snapshot(24, 3, 'c').unwrap();
    ///
    /// // `24w03b` is less than `24w03c`
    /// assert_eq!(snapshot_1.try_compare(&snapshot_2), Some(std::cmp::Ordering::Less));
    ///
    /// // `24w03b` is equal to `24w03b`
    /// assert_eq!(snapshot_1.try_compare(&snapshot_1), Some(std::cmp::Ordering::Equal));
    ///
    /// // `24w03c` is greater than `24w03b`
    /// assert_eq!(snapshot_2.try_compare(&snapshot_1), Some(std::cmp::Ordering::Greater));
    ///
    /// // A release and snapshot are not comparable.
    /// assert_eq!(release_1.try_compare(&snapshot_1), None);
    /// assert_eq!(release_2.try_compare(&snapshot_2), None);
    ///
    /// assert_eq!(snapshot_1.try_compare(&release_1), None);
    /// assert_eq!(snapshot_2.try_compare(&release_2), None);
    /// ```
    #[must_use]
    pub fn try_compare(&self, other: &Self) -> Option<Ordering> {
        // If the discriminants are not the same, the versions are not comparable.
        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            (MinecraftVersion::Release(a), MinecraftVersion::Release(b))
            | (MinecraftVersion::ReleaseCandidate(a), MinecraftVersion::ReleaseCandidate(b))
            | (MinecraftVersion::PreRelease(a), MinecraftVersion::PreRelease(b))
            | (MinecraftVersion::Snapshot(a), MinecraftVersion::Snapshot(b)) => Some(a.cmp(b)),
            (MinecraftVersion::Other(a), MinecraftVersion::Other(b)) => Some(a.cmp(b)),
            _ => unreachable!(),
        }
    }

    /// Returns the version as a [`String`].
    ///
    /// # Examples
    /// ```rust
    /// use froglight_definitions::MinecraftVersion;
    ///
    /// let version = MinecraftVersion::new_release_candidate(1, 20, 0, 2).unwrap();
    /// assert_eq!(version.as_long_string(), "1.20.0-rc2");
    ///
    /// let version = MinecraftVersion::new_release(1, 20, 0);
    /// assert_eq!(version.as_long_string(), "1.20.0");
    ///
    /// let version = MinecraftVersion::new_pre_release(1, 20, 4, 2).unwrap();
    /// assert_eq!(version.as_long_string(), "1.20.4-pre2");
    ///
    /// let version = MinecraftVersion::new_release(1, 20, 4);
    /// assert_eq!(version.as_long_string(), "1.20.4");
    ///
    /// let version = MinecraftVersion::new_snapshot(24, 3, 'b').unwrap();
    /// assert_eq!(version.as_long_string(), "24w03b");
    /// ```
    #[must_use]
    pub fn as_long_string(&self) -> String { self.to_string() }

    /// Returns the version as a short [`String`].
    ///
    /// This will remove the patch version if it is `0`.
    ///
    /// # Examples
    /// ```rust
    /// use froglight_definitions::MinecraftVersion;
    ///
    /// let version = MinecraftVersion::new_release_candidate(1, 20, 0, 2).unwrap();
    /// assert_eq!(version.as_short_string(), "1.20-rc2");
    ///
    /// let version = MinecraftVersion::new_release(1, 20, 0);
    /// assert_eq!(version.as_short_string(), "1.20");
    ///
    /// let version = MinecraftVersion::new_pre_release(1, 20, 4, 2).unwrap();
    /// assert_eq!(version.as_short_string(), "1.20.4-pre2");
    ///
    /// let version = MinecraftVersion::new_release(1, 20, 4);
    /// assert_eq!(version.as_short_string(), "1.20.4");
    ///
    /// let version = MinecraftVersion::new_snapshot(24, 3, 'b').unwrap();
    /// assert_eq!(version.as_short_string(), "24w03b");
    /// ```
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn as_short_string(&self) -> String {
        let mut string = self.to_string();

        if !matches!(self, Self::Snapshot(_) | Self::Other(_)) {
            let mut split = string.split('.');
            let major = split.next().unwrap();
            let minor = split.next().unwrap();

            let patch = split.next().unwrap();
            if let Some(stripped) = patch.strip_prefix('0') {
                string = format!("{major}.{minor}{stripped}");
            }
        }

        string
    }
}

impl<T: AsRef<str>> From<T> for MinecraftVersion {
    fn from(value: T) -> Self { value.as_ref().parse().unwrap() }
}

impl FromStr for MinecraftVersion {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(ver) = regex::parse_release(s) {
            Ok(ver)
        } else if let Some(ver) = regex::parse_release_candidate(s) {
            Ok(ver)
        } else if let Some(ver) = regex::parse_pre_release(s) {
            Ok(ver)
        } else if let Some(ver) = regex::parse_snapshot(s) {
            Ok(ver)
        } else {
            Ok(MinecraftVersion::new_other(s))
        }
    }
}

impl std::fmt::Debug for MinecraftVersion {
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

impl std::fmt::Display for MinecraftVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Release(ver) => {
                write!(f, "{}.{}.{}", ver.major, ver.minor, ver.patch)
            }
            Self::ReleaseCandidate(ver) => {
                write!(f, "{}.{}.{}-rc{}", ver.major, ver.minor, ver.patch, ver.pre)
            }
            Self::PreRelease(ver) => {
                write!(f, "{}.{}.{}-pre{}", ver.major, ver.minor, ver.patch, ver.pre)
            }
            Self::Snapshot(ver) => {
                write!(f, "{}w{:02}{}", ver.major, ver.minor, ver.pre)
            }
            Self::Other(ver) => f.write_str(ver),
        }
    }
}

impl Serialize for MinecraftVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for MinecraftVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VersionVisitor;
        impl<'de> serde::de::Visitor<'de> for VersionVisitor {
            type Value = MinecraftVersion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid Minecraft version")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Self::visit_str(self, &v)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(v.parse().unwrap())
            }
        }

        deserializer.deserialize_string(VersionVisitor)
    }
}
