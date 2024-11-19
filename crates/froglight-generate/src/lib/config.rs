use derive_more::derive::{Deref, DerefMut};
use froglight_parse::Version;
use serde::{Deserialize, Serialize};

/// The configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
pub struct Config {
    /// A list of versions to generate.
    pub version: Vec<VersionTuple>,
}

/// A pair of [`Versions`](Version).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionTuple {
    /// The version used by name.
    pub base: Version,
    /// The version used to generate data.
    pub target: Version,
}

impl VersionTuple {
    /// Create a new [`VersionTuple`].
    #[must_use]
    pub const fn new(base: Version, target: Version) -> Self { Self { base, target } }
}

impl From<(Version, Version)> for VersionTuple {
    fn from((base, target): (Version, Version)) -> Self { Self { base, target } }
}
impl From<VersionTuple> for (Version, Version) {
    fn from(VersionTuple { base, target }: VersionTuple) -> Self { (base, target) }
}
