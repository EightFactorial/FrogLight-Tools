use convert_case::{Case, Casing};
use froglight_generate::{
    modules::packet::{File, Result, State},
    CliArgs, DataMap, PacketGenerator,
};
use froglight_parse::file::protocol::ProtocolTypeMap;
use syn::Item;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

use super::{process::ProcessResult, GeneratedTypes};

pub(super) async fn generate_common(
    datamap: &DataMap,
    args: &CliArgs,
) -> anyhow::Result<GeneratedTypes> {
    let mut generated = GeneratedTypes::default();

    // Get all of the protocol types that are identical across all versions
    {
        // For all types in the first version
        if let Some(data) = datamap.version_data.values().next() {
            for (proto_name, proto_data) in data
                .proto
                .types
                .iter()
                .filter(|(name, data)| !name.starts_with("packet") && !data.is_native())
            {
                // If all versions contain it *and* it's identical
                if datamap.version_data.values().all(|data| {
                    if let Some(data) = data.proto.types.get(proto_name) {
                        data == proto_data
                    } else {
                        false
                    }
                }) {
                    // Generate the type
                    generated.insert(
                        proto_name.to_string(),
                        (get_item_module(proto_name), proto_name.to_case(Case::Pascal)),
                    );
                }
            }
        }

        // For any types that are alone, rename the module
        let cloned = generated.clone();
        for (item, (_, item_name)) in cloned.iter().filter(|(item_a, (module_a, _))| {
            cloned
                .iter()
                .all(|(item_b, (module_b, _))| (*item_a == item_b) || (module_a != module_b))
        }) {
            generated.get_mut(item).unwrap().0 = item_name.to_case(Case::Snake);
        }
    }

    // Delete all previously generated files
    let dir = args.dir.join("crates/froglight-protocol/src/generated/common/");
    if dir.exists() {
        tracing::warn!("PacketGenerator: Removing directory \"{}\"", dir.display());
        tokio::fs::remove_dir_all(&dir).await?;
    }

    // Generate the types
    if let Some(data) = datamap.version_data.values().next() {
        for (protocol, (module, item)) in generated.iter() {
            generate_common_items(protocol, module, item, &data.proto.types, args).await?;
        }
    }

    Ok(generated)
}

/// Get the module name for a given item name.
fn get_item_module(proto_name: &str) -> String {
    let mut name = String::new();

    for (index, c) in proto_name.chars().enumerate() {
        if index == 0 {
            name.push(c.to_ascii_lowercase());
        } else if c.is_ascii_lowercase() || !c.is_ascii() {
            name.push(c);
        } else {
            break;
        }
    }

    name.to_case(Case::Snake)
}

async fn generate_common_items(
    protocol: &str,
    module: &str,
    item: &str,
    types: &ProtocolTypeMap,
    args: &CliArgs,
) -> anyhow::Result<()> {
    let protocol = types.get(protocol).expect("Protocol type not found?");

    // Recursively generate any needed items
    let mut file = File::new();
    if let Result::Err(err) = PacketGenerator::generate_type(
        &State::new().with_item(item).with_target("_"),
        protocol,
        &mut file,
    ) {
        tracing::error!("Error generating item \"{item}\": {err}");
        return Err(err);
    }

    let mut replacements = Vec::new();

    // Clean up the generated items
    let mut file = file.into_inner();
    let mut processed = Vec::with_capacity(file.items.len());
    for item in file.items {
        let item_name = match &item {
            Item::Enum(item) => item.ident.to_string().to_case(Case::Pascal),
            Item::Struct(item) => item.ident.to_string().to_case(Case::Pascal),
            _ => continue,
        };
        match super::process::process_item(item) {
            ProcessResult::Replaced(ident) => {
                replacements.push((item_name, ident.to_string()));
            }
            ProcessResult::Processed(item) => processed.push(item),
            ProcessResult::Removed => {}
        }
    }
    file.items = processed;

    // Unparse the generated file, returning early if it's empty
    let mut content = prettyplease::unparse(&file);
    if content.is_empty() {
        return Ok(());
    }

    // Return early if the content contains the word "Unsupported"
    if content.contains("Unsupported") {
        tracing::warn!("PacketGenerator: Skipping item \"{item}\" due to unsupported types");
        return Ok(());
    }

    // Manually edit the content to fix some issues
    content = content.replace(" type_", " kind");
    for (existing, replacement) in replacements {
        tracing::info!("PacketGenerator: Replacing \"{existing}\" with \"{replacement}\"");
        content = content.replace(&existing, &replacement);
    }

    // Write the file to disk
    let file_path = args
        .dir
        .join("crates/froglight-protocol/src/generated/common/")
        .join(format!("{module}.rs"));
    if !file_path.exists() {
        tracing::warn!("PacketGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }

    let mut file = OpenOptions::new().write(true).create(true).append(true).open(file_path).await?;
    file.write_all(content.as_bytes()).await?;

    Ok(())
}
