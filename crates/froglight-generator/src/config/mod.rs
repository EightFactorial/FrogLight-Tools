use froglight_data::Version;
use serde::Deserialize;
use tracing::trace;

use crate::command::GeneratorArgs;

/// The configuration for [`FrogLight-Generator`](crate).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct GeneratorConfig {
    pub(crate) versions: Vec<SupportedVersion>,
}

impl GeneratorConfig {
    /// Load the [`GeneratorConfig`] from the command line arguments.
    ///
    /// # Errors
    /// - If the config file cannot be found or read
    pub(crate) async fn from_args(args: &GeneratorArgs) -> anyhow::Result<Self> {
        // Get the path to the config file
        let config_path = args.directory.join(&args.config);
        trace!("Reading GeneratorConfig from `{}`", config_path.display());

        // Read the config file
        let config = tokio::fs::read_to_string(config_path).await?;
        Ok(toml::from_str(&config)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct SupportedVersion {
    // The base version of the game
    pub(crate) base_version: Version,
    // The jar to use for extracting the game data
    pub(crate) jar_version: Version,
}
