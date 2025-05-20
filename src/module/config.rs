use std::path::PathBuf;

use clap::Parser;
use froglight_dependency::{
    container::{Dependency, DependencyContainer, SharedDependencies},
    version::Version,
};
use serde::Deserialize;

use super::{Blocks, Items, Packets};

#[derive(Debug, Clone, PartialEq, Eq, Parser, Dependency)]
#[dep(retrieve = Self::parse)]
pub(crate) struct ToolArgs {
    /// Path to the configuration file
    #[clap(short, long)]
    pub(crate) config: PathBuf,
    /// The list of modules to run
    ///
    /// If empty, all modules will be run
    #[clap(name = "module", short, long)]
    pub(crate) modules: Vec<String>,
}

impl ToolArgs {
    /// The default set of modules to run if none are specified
    const DEFAULT: &[&str] = &[Blocks::MODULE_NAME, Items::MODULE_NAME, Packets::MODULE_NAME];

    #[expect(clippy::unused_async)]
    async fn parse(_: &mut DependencyContainer) -> anyhow::Result<Self> {
        let mut args: Self = Parser::parse();

        // If no modules are specified, run the defaults
        if args.modules.is_empty() {
            args.modules.extend(Self::DEFAULT.iter().map(ToString::to_string));
        }

        Ok(args)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Dependency, Deserialize)]
#[dep(retrieve = Self::parse)]
pub(crate) struct ToolConfig {
    pub(crate) versions: Vec<Version>,
    #[serde(skip)]
    pub(crate) modules: Vec<String>,
}

impl ToolConfig {
    /// Retrieve the configuration from the dependency container
    #[inline]
    pub(crate) async fn get(deps: &SharedDependencies) -> anyhow::Result<Self> {
        deps.write().await.get_or_retrieve::<Self>().await.cloned()
    }

    async fn parse(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let ToolArgs { config, modules } = deps.get_or_retrieve::<ToolArgs>().await?.clone();

        let path = if tokio::fs::try_exists(&config).await? {
            config
        } else {
            tracing::debug!("Configuration not found in current directory...");
            deps.cache.parent().unwrap().parent().unwrap().join(config)
        };

        tracing::debug!("Loading configuration from: \"{}\"", path.display());
        let file = tokio::fs::read_to_string(path).await?;

        Ok(Self { versions: toml_edit::de::from_str::<Self>(&file)?.versions, modules })
    }
}
