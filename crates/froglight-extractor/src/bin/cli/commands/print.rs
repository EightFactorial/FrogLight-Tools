use async_zip::tokio::read::fs::ZipFileReader;
use froglight_data::VersionManifest;
use froglight_extractor::jar;
use tracing::error;

use super::{Command, SubCommand};

pub(crate) async fn print(command: &Command, manifest: &VersionManifest) -> anyhow::Result<()> {
    let SubCommand::Print(args) = &command.subcommand else { unreachable!() };

    if !std::path::Path::new(&args.class)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("class"))
    {
        let msg = format!("Invalid class file: `{}`, must have a `.class` extension", args.class);
        error!(msg);
        anyhow::bail!(msg);
    }

    // Load the jar file
    let jar_path =
        jar::get_mapped_jar(&command.version, manifest, &command.cache, command.refresh).await;
    let jar = ZipFileReader::new(jar_path).await.expect("Failed to read jar file");

    // Search for the class
    let file_count = jar.file().entries().len();
    for file_index in 0..file_count {
        let mut entry = jar.reader_with_entry(file_index).await.expect("Failed to read jar file");

        if let Ok(str) = entry.entry().filename().as_str() {
            if args.class == str {
                // Read the class data
                let mut data = Vec::new();
                entry.read_to_end_checked(&mut data).await.expect("Failed to read class file");

                // Parse the class file
                let classfile = cafebabe::parse_class(&data).expect("Failed to parse class file");

                // Return the class information
                match &command.output {
                    Some(path) => {
                        // Write the result to the output file
                        serde_json::to_writer_pretty(
                            std::fs::File::create(path).expect("Failed to create output file"),
                            &format!("{classfile:#?}"),
                        )
                        .expect("Failed to write output to file");
                    }
                    None => {
                        // Write the result to stdout
                        println!("{classfile:#?}");
                    }
                }

                return Ok(());
            }
        }
    }

    let msg = format!("Class `{}` not found in jar", args.class);
    error!(msg);
    anyhow::bail!(msg);
}
