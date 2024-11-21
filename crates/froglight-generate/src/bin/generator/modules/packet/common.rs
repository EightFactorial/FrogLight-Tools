use convert_case::{Case, Casing};
use froglight_generate::{
    modules::packet::{File, Result, State},
    CliArgs, DataMap, PacketGenerator,
};
use froglight_parse::file::protocol::ProtocolTypeMap;
use syn::{Attribute, GenericArgument, Ident, Item, PathArguments, Type};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

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

                    for field in &mut variant.fields {
                        if let Type::Path(path) = &mut field.ty {
                            if let Some(segment) = path.path.segments.first_mut() {
                                if let Some(attr) = edit_item_field(segment) {
                                    field.attrs.push(attr);
                                }
                            }
                        }
                    }
                }
            }
            Item::Struct(item) => {
                item.ident =
                    Ident::new(&item.ident.to_string().to_case(Case::Pascal), item.ident.span());

                for field in &mut item.fields {
                    if let Type::Path(path) = &mut field.ty {
                        if let Some(segment) = path.path.segments.first_mut() {
                            if let Some(attr) = edit_item_field(segment) {
                                field.attrs.push(attr);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Unparse the generated file, skipping if it's empty
    let mut content = prettyplease::unparse(&file);
    if content.is_empty() {
        return Ok(());
    }

    content = content.replace(" type_", " kind");

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

const IGNORE_TYPES: &[&str] =
    &["bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "isize", "f32", "f64"];

fn edit_item_field(segment: &mut syn::PathSegment) -> Option<Attribute> {
    // Check the type's generic arguments
    let mut returned = None;
    if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
        for arg in &mut args.args {
            if let GenericArgument::Type(Type::Path(path)) = arg {
                if let Some(segment) = path.path.segments.first_mut() {
                    if let Some(attr) = edit_item_field(segment) {
                        returned = Some(attr);
                    }
                }
            }
        }
    }

    // Check if the field type is a varint or varlong
    let segment_type = segment.ident.to_string();
    match segment_type.as_str() {
        "varint" => {
            segment.ident = Ident::new("u32", segment.ident.span());
            return Some(syn::parse_quote!(#[frog(var)]));
        }
        "varlong" => {
            segment.ident = Ident::new("u64", segment.ident.span());
            return Some(syn::parse_quote!(#[frog(var)]));
        }
        _ => {}
    }

    // Format the field type to match rust conventions
    if !IGNORE_TYPES.contains(&segment_type.as_str()) {
        segment.ident =
            Ident::new(&segment.ident.to_string().to_case(Case::Pascal), segment.ident.span());
    }

    returned
}
