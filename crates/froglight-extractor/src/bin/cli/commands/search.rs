use froglight_data::VersionManifest;
use froglight_extractor::classmap::ClassMap;
use tokio::io::AsyncWriteExt;

use super::{ExtractCommand, ExtractSubCommand};

pub(crate) async fn search(
    command: &ExtractCommand,
    manifest: &VersionManifest,
) -> anyhow::Result<()> {
    let ExtractSubCommand::Search(args) = &command.subcommand else { unreachable!() };

    let classmap =
        ClassMap::new(&command.version, manifest, &command.cache, command.refresh).await?;
    let mut filtered_map = ClassMap::empty();

    for (key, value) in classmap.into_iter() {
        // Check if the key contains the query
        if key.contains(&args.query) {
            filtered_map.insert(key, value);
            continue;
        }

        // Parse the class
        let class = value.parse();

        // Check if the class name contains the query
        if class.this_class.contains(&args.query) {
            filtered_map.insert(key, value);
        } else if let Some(super_class) = class.super_class {
            // Check if the super class contains the query
            if super_class.contains(&args.query) {
                filtered_map.insert(key, value);
            }
        } else if class.interfaces.iter().any(|interface| interface.contains(&args.query)) {
            // Check if any of the interfaces contain the query
            filtered_map.insert(key, value);
        } else if class.fields.iter().any(|field| field.name.contains(&args.query)) {
            // Check if any of the fields contain the query
            filtered_map.insert(key, value);
        } else if class.methods.iter().any(|method| method.name.contains(&args.query)) {
            // Check if any of the methods contain the query
            filtered_map.insert(key, value);
        }
    }

    // Return the class information
    if let Some(output) = &command.output {
        // Write the result to the output file.
        tokio::fs::write(output, format!("{filtered_map:#?}")).await?;
    } else {
        // Write the result to stdout.
        tokio::io::stdout().write_all(format!("{filtered_map:#?}").as_bytes()).await?;
    }

    Ok(())
}
