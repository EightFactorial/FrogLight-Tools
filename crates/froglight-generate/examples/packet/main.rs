//! TODO

use std::path::{Path, PathBuf};

use froglight_generate::{CliArgs, DataMap, PacketGenerator};
use froglight_parse::{
    file::protocol::{ProtocolStatePackets, ProtocolTypeMap},
    Version,
};
use syn::{File, GenericArgument, Item, PathArguments, Type, TypePath};

/// The version to generate packets for.
const GENERATE_VERSION: Version = Version::new_release(1, 21, 1);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (args, _) = CliArgs::parse().await?;
    let datamap =
        DataMap::new_from(&args.cache.unwrap(), &[GENERATE_VERSION], args.redownload).await?;

    if let Some(dataset) = datamap.version_data.get(&GENERATE_VERSION) {
        let output = PathBuf::from(file!()).parent().unwrap().to_path_buf().join("generated");
        tracing::info!("Version: {GENERATE_VERSION}");

        generate_types(&output, &dataset.proto.types).await?;
        for (state, packets) in dataset.proto.packets.iter() {
            generate_packets(state, "clientbound", &output, &packets.clientbound).await?;
            generate_packets(state, "serverbound", &output, &packets.serverbound).await?;
        }
    }

    Ok(())
}

async fn generate_types(directory: &Path, types: &ProtocolTypeMap) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(directory).await?;
    let (generated, _) = PacketGenerator::generate_types(types);

    // If there are no types, skip writing to the file
    if !generated.items.is_empty() {
        let content = prettyplease::unparse(&generated);
        let output = directory.join("protocol_types.rs");
        tracing::info!("Writing: {}", output.display());
        tokio::fs::write(output, &content).await?;
    }

    Ok(())
}

async fn generate_packets(
    state: &str,
    direction: &str,
    directory: &Path,
    packets: &ProtocolStatePackets,
) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(directory).await?;
    let (mut generated, _) = PacketGenerator::generate_packets(packets);

    // Filter out unsupported items
    let cloned = generated.clone();
    generated.items.retain(|item| {
        if item_supported(item, &cloned) {
            true
        } else {
            tracing::warn!(
                "Removing unsupported item: \"{}\"",
                match item {
                    Item::Enum(item_enum) => item_enum.ident.to_string(),
                    Item::Struct(item_struct) => item_struct.ident.to_string(),
                    _ => "Unknown".to_string(),
                }
            );
            false
        }
    });

    // If there are no packets, skip writing to the file
    if !generated.items.is_empty() {
        let content = prettyplease::unparse(&generated);
        let output = directory.join(format!("{state}_{direction}.rs"));
        tracing::info!("Writing: {}", output.display());
        tokio::fs::write(output, &content).await?;
    }

    Ok(())
}

/// Check if the [`Item`] is supported.
///
/// Recursively checks *all* fields, variants, and types.
#[must_use]
fn item_supported(item: &Item, file: &File) -> bool {
    match item {
        Item::Enum(item_enum) => item_enum.variants.iter().all(|variant| {
            variant.fields.iter().all(|field| {
                if let Type::Path(type_path) = &field.ty {
                    path_supported(type_path, file)
                } else {
                    true
                }
            })
        }),
        Item::Struct(item_struct) => item_struct.fields.iter().all(|field| {
            if let Type::Path(type_path) = &field.ty {
                path_supported(type_path, file)
            } else {
                true
            }
        }),
        _ => true,
    }
}

/// Check if the [`TypePath`] is supported.
///
/// Recursively checks *all* path segments and generic arguments.
#[must_use]
fn path_supported(path: &TypePath, file: &File) -> bool {
    path.path.segments.iter().all(|segment| {
        if segment.ident == "Unsupported" {
            false
        } else if let Some(item) = file.items.iter().find(|item| match item {
            Item::Enum(item_enum) => item_enum.ident == segment.ident,
            Item::Struct(item_struct) => item_struct.ident == segment.ident,
            _ => false,
        }) {
            item_supported(item, file)
        } else {
            match &segment.arguments {
                PathArguments::None => true,
                PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                    angle_bracketed_generic_arguments.args.iter().all(|arg| {
                        if let GenericArgument::Type(Type::Path(type_path)) = arg {
                            path_supported(type_path, file)
                        } else {
                            true
                        }
                    })
                }
                PathArguments::Parenthesized(parenthesized_generic_arguments) => {
                    parenthesized_generic_arguments.inputs.iter().all(|arg| {
                        if let Type::Path(type_path) = arg {
                            path_supported(type_path, file)
                        } else {
                            true
                        }
                    })
                }
            }
        }
    })
}
