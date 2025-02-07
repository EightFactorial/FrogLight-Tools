#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use froglight_dependency::container::SharedDependencies;
use froglight_extract::module::ExtractModule;
use module::ToolConfig;

mod module;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    froglight_extract::cmd::logging();

    // Setup the shared dependencies and modules
    let deps = SharedDependencies::from_rust_env();
    let modules = ExtractModule::map();

    // Load the command line arguments and configuration file
    let config = deps.write().await.get_or_retrieve::<ToolConfig>().await?.clone();

    // Run the modules for each version
    for version in config.versions {
        tracing::info!("Version: {version}");
        for module in &config.modules {
            if let Some(extract) = modules.get(module.as_str()) {
                tracing::info!("Running module: {}", module);
                extract.run(&version, &mut *deps.write().await).await?;
            } else {
                tracing::error!("Unknown module: {}", module);
            }
        }
    }

    Ok(())
}
