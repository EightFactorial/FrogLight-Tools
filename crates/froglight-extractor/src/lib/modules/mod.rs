//! Modules for extracting data from a source.

use std::path::Path;

use froglight_data::Version;
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, EnumString};

mod assets;
pub use assets::AssetModule;

mod debug;
pub use debug::DebugModule;

mod info;
pub use info::InfoModule;

mod protocol;
pub use protocol::ProtocolModule;

use crate::classmap::ClassMap;

/// A module to use for extracting data.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[allow(missing_docs)]
pub enum ExtractModule {
    Debug(DebugModule),
    Info(InfoModule),
    Assets(AssetModule),
    Protocol(ProtocolModule),
}

impl ExtractModule {
    /// Run the extraction on the given classmap.
    #[allow(clippy::missing_errors_doc)]
    pub async fn extract(
        &self,
        version: &Version,
        classmap: &ClassMap,
        cache: &Path,
        output: &mut serde_json::Value,
    ) -> anyhow::Result<()> {
        match self {
            Self::Assets(module) => module.extract(version, classmap, cache, output).await,
            Self::Debug(module) => module.extract(version, classmap, cache, output).await,
            Self::Info(module) => module.extract(version, classmap, cache, output).await,
            Self::Protocol(module) => module.extract(version, classmap, cache, output).await,
        }
    }
}

/// A trait for extracting data from a classmap.
trait Extract {
    /// Run the extraction on the given classmap.
    fn extract(
        &self,
        version: &Version,
        classmap: &ClassMap,
        cache: &Path,
        output: &mut serde_json::Value,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}
