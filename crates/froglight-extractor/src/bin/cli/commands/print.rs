use async_zip::tokio::read::fs::ZipFileReader;
use froglight_data::VersionManifest;
use froglight_extractor::jar;
use tokio::io::AsyncWriteExt;

use super::{ExtractCommand, ExtractSubCommand};

pub(crate) async fn print(
    command: &ExtractCommand,
    manifest: &VersionManifest,
) -> anyhow::Result<()> {
    let ExtractSubCommand::Print(args) = &command.subcommand else { unreachable!() };

    if !std::path::Path::new(&args.class)
        .extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("class"))
    {
        anyhow::bail!("Invalid class file: `{}`, must have a `.class` extension", args.class);
    }

    // Load the jar file
    let jar_path =
        jar::get_mapped_jar(&command.version, manifest, &command.cache, command.refresh).await?;
    let jar = ZipFileReader::new(jar_path).await?;

    // Search for the class
    let file_count = jar.file().entries().len();
    for file_index in 0..file_count {
        let mut entry = jar.reader_with_entry(file_index).await?;

        if let Ok(str) = entry.entry().filename().as_str() {
            if args.class == str {
                // Read the class data
                let mut data = Vec::new();
                entry.read_to_end_checked(&mut data).await?;

                // Parse the class file
                let classfile = cafebabe::parse_class(&data)?;

                // Return the class information
                if let Some(output) = &command.output {
                    // Write the result to the output file.
                    tokio::fs::write(output, format!("{classfile:#?}")).await?;
                } else {
                    // Write the result to stdout.
                    tokio::io::stdout().write_all(format!("{classfile:#?}").as_bytes()).await?;
                }

                return Ok(());
            }
        }
    }

    anyhow::bail!("Class `{}` not found in jar", args.class);
}
