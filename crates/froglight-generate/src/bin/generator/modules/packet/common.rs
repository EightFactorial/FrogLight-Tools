use convert_case::{Case, Casing};
use froglight_generate::{
    modules::packet::{File, Result, State},
    CliArgs, DataMap, PacketGenerator,
};
use froglight_parse::file::protocol::ProtocolTypeMap;
use syn::{GenericArgument, Ident, Item, PathArguments, Type};

use super::GeneratedTypes;

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
                    generated.insert(proto_name.to_string(), get_item_path(proto_name));
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

fn get_item_path(proto_name: &str) -> (String, String) {
    (get_item_module(proto_name), proto_name.to_case(Case::Pascal))
}

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

    let mut file = File::new();
    let state = State::new().with_item(item);

    // Recursively generate any needed items
    if let Result::Err(err) =
        PacketGenerator::generate_type(&state.with_target("_"), protocol, &mut file)
    {
        tracing::error!("Error generating item \"{item}\": {err}");
        return Err(err);
    }

    // Manually edit generated items
    let mut file = file.into_inner();
    for item in &mut file.items {
        match item {
            Item::Enum(item) => {
                item.ident =
                    Ident::new(&item.ident.to_string().to_case(Case::Pascal), item.ident.span());

                for variant in &mut item.variants {
                    variant.ident = Ident::new(
                        &variant.ident.to_string().to_case(Case::Pascal),
                        variant.ident.span(),
                    );
                }
            }
            Item::Struct(item) => {
                item.ident =
                    Ident::new(&item.ident.to_string().to_case(Case::Pascal), item.ident.span());

                for field in &mut item.fields {
                    if let Some(ident) = field.ident.as_mut() {
                        *ident = Ident::new(&ident.to_string().to_case(Case::Snake), ident.span());
                    }

                    if let Type::Path(path) = &mut field.ty {
                        if let Some(segment) = path.path.segments.iter_mut().last() {
                            segment.ident = Ident::new(
                                &segment.ident.to_string().to_case(Case::Pascal),
                                segment.ident.span(),
                            );

                            if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments
                            {
                                for arg in &mut arguments.args {
                                    if let GenericArgument::Type(Type::Path(path)) = arg {
                                        if let Some(segment) = path.path.segments.iter_mut().last()
                                        {
                                            segment.ident = Ident::new(
                                                &segment.ident.to_string().to_case(Case::Pascal),
                                                segment.ident.span(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Unparse the generated file
    let content = prettyplease::unparse(&file);

    // Write the file to disk
    let file_path = args
        .dir
        .join("crates/froglight-protocol/src/generated/common/")
        .join(format!("{module}.rs"));
    if !file_path.exists() {
        tracing::warn!("PacketGenerator: Creating file \"{}\"", file_path.display());
        tokio::fs::create_dir_all(file_path.parent().unwrap()).await?;
    }
    tokio::fs::write(file_path, &content).await?;

    Ok(())
}
