use froglight_data::VersionManifest;
use froglight_extractor::{classmap::ClassMap, modules::ExtractModule};
use strum::IntoEnumIterator;
use tracing::{error, info};

use super::{ExtractCommand, ExtractSubCommand};

pub(crate) async fn extract(
    command: &ExtractCommand,
    manifest: &VersionManifest,
) -> anyhow::Result<()> {
    let ExtractSubCommand::Extract(args) = &command.subcommand else { unreachable!() };

    // Get the modules to run
    let mut modules = args.modules.clone();
    if modules.is_empty() {
        modules = ExtractModule::iter().collect();
    } else {
        // TODO: Add dependency resolution.
        modules.sort();
    }
    info!("Modules: {modules:?}");

    // Create the classmap
    let classmap =
        ClassMap::new(&command.version, manifest, &command.cache, command.refresh).await?;

    // Run the modules
    let mut json = serde_json::Value::default();
    for module in modules {
        if let Err(err) =
            module.extract(&command.version, &classmap, &command.cache, &mut json).await
        {
            error!("Error during {module:?}: `{err}`");
        }
    }

    // Print the result
    if let Some(output) = &command.output {
        // Write the result to the output file.
        serde_json::to_writer_pretty(std::fs::File::create(output)?, &json)?;
    } else {
        // Write the result to stdout.
        serde_json::to_writer_pretty(std::io::stdout(), &json)?;
    }

    Ok(())
}
