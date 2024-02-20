use std::sync::Arc;

use cargo_metadata::Metadata as Workspace;
use froglight_data::Version;
use froglight_extractor::{classmap::ClassMap, modules::ExtractModule};
use hashbrown::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use tokio::task::JoinSet;
use tracing::{debug, error, info};

use crate::{command::GeneratorArgs, config::GeneratorConfig};

mod network;
use network::NetworkModule;

mod protocol;
use protocol::ProtocolModule;

mod util;
use util::DataBundle;

/// Run all generator modules
pub(super) async fn run(
    args: GeneratorArgs,
    config: GeneratorConfig,
    workspace: Workspace,
) -> anyhow::Result<()> {
    // Get the version manifest
    let target = workspace.target_directory.as_std_path();
    let manifest = froglight_extractor::manifest::version_manifest(target, false).await?;

    // Collect data for all versions
    let mut version_data: HashMap<Version, (ClassMap, serde_json::Value)> =
        HashMap::with_capacity(config.versions.len());
    for version in &config.versions {
        debug!("Gathering data for: {} ({})", version.base_version, version.jar_version);

        // Create a classmap
        let classmap = ClassMap::new(&version.jar_version, &manifest, target, false).await?;

        // Extract data for all modules
        let mut extracted = serde_json::Value::default();
        for module in ExtractModule::iter().filter(|m| !matches!(m, ExtractModule::Assets(_))) {
            module.extract(&version.jar_version, &classmap, target, &mut extracted).await?;
        }

        version_data.insert(version.base_version.clone(), (classmap, extracted));
    }

    // Bundle all data to make it easier to pass around
    let bundle = Arc::new(DataBundle::new(args, config, workspace, version_data));

    // Run all modules in parallel
    let mut task_set = JoinSet::new();
    for module in GeneratorModule::iter() {
        debug!("Running: {module:?}");
        task_set.spawn(module.generate(bundle.clone()));
    }

    // Wait for all tasks to complete
    while let Some(result) = task_set.join_next().await {
        match result {
            Err(err) => error!("Error joining task: `{err}`"),
            Ok(Err(err)) => error!("Error running generator: `{err}`"),
            Ok(Ok(())) => {}
        }
    }

    info!("Done!");

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum GeneratorModule {
    Network(NetworkModule),
    Protocol(ProtocolModule),
}

impl GeneratorModule {
    async fn generate(self, bundle: Arc<DataBundle>) -> anyhow::Result<()> {
        match self {
            Self::Network(module) => module.generate(bundle).await,
            Self::Protocol(module) => module.generate(bundle).await,
        }
    }
}

trait Generate {
    fn generate(
        &self,
        bundle: Arc<DataBundle>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}
