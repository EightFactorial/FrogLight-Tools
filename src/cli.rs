//! TODO

use std::str::FromStr;

use clap::Parser;
use serde::{Deserialize, Serialize};

/// Command line arguments.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Parser)]
pub struct ToolArguments {
    /// The version
    #[clap(alias = "config")]
    pub version: VersionOrConfig,
}

/// The version to use or the configuration file for which version(s) to use.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VersionOrConfig {
    /// The version to use.
    Version(String),
    /// The configuration file for which version(s) to use.
    Config(VersionConfig),
}
impl FromStr for VersionOrConfig {
    type Err = anyhow::Error;
    #[expect(clippy::case_sensitive_file_extension_comparisons)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(".toml") {
            Ok(Self::Config(serde_json::from_reader(std::fs::File::open(s)?)?))
        } else {
            Ok(Self::Version(s.to_string()))
        }
    }
}

/// The configuration file for which version(s) to use.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionConfig {
    version: VersionPair,
}
impl std::ops::Deref for VersionConfig {
    type Target = VersionPair;
    fn deref(&self) -> &Self::Target { &self.version }
}

/// A pair of name and data versions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionPair {
    /// The name of the version.
    pub name: String,
    /// The version whose data to use.
    pub data: String,
}
