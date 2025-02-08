#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use froglight_dependency::container::SharedDependencies;
use froglight_extract::module::ExtractModule;
use module::ToolConfig;

mod class_helper;
mod module;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    froglight_extract::cmd::logging();

    // Setup the shared dependencies and modules
    let deps = SharedDependencies::from_rust_env();
    let modules = ExtractModule::map();

    // Parse the command line arguments and load the configuration file
    let config = ToolConfig::get(&deps).await?;

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
