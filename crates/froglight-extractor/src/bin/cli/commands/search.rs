use froglight_data::VersionManifest;

use super::{Command, SubCommand};
use crate::classmap::ClassMap;

pub(crate) async fn search(command: &Command, manifest: &VersionManifest) -> anyhow::Result<()> {
    let SubCommand::Search(args) = &command.subcommand else { unreachable!() };

    let classmap = ClassMap::new(command, manifest).await;
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

    match &command.output {
        Some(path) => {
            // Write the result to the output file
            serde_json::to_writer_pretty(
                std::fs::File::create(path).expect("Failed to create output file"),
                &format!("{filtered_map:#?}"),
            )
            .expect("Failed to write output to file");
        }
        None => {
            // Write the result to stdout
            println!("{filtered_map:#?}");
        }
    }

    Ok(())
}
