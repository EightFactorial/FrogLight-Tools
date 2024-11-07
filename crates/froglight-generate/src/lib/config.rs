use derive_more::derive::{Deref, DerefMut};
use froglight_parse::Version;
use serde::{Deserialize, Serialize};

/// The configuration file.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
pub struct Config {
    version: Vec<VersionTuple>,
}

/// A pair of [`Versions`](Version).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionTuple {
    /// The version used by name.
    pub base: Version,
    /// The version used to generate data.
    pub target: Version,
}
