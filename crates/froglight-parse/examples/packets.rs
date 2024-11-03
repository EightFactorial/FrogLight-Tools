//! An example of generating packets from the protocol.
//!
//! Does not generate perfect code, but it's not meant to.

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use convert_case::{Case, Casing};
use froglight_parse::{
    file::{
        protocol::{ArrayArgs, BufferArgs, ContainerArg, ProtocolType, ProtocolTypeArgs},
        DataPath, FileTrait, VersionProtocol,
    },
    Version,
};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Field, FieldMutability, Fields, FieldsNamed, Generics, Ident,
    ItemStruct, Visibility,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache = target_dir().await;
    let client = reqwest::Client::new();

    let v1_21_1 = Version::new_release(1, 21, 1);

    let datapaths = DataPath::fetch(&v1_21_1, &cache, &(), false, &client).await?;
    let protocol = VersionProtocol::fetch(&v1_21_1, &cache, &datapaths, false, &client).await?;

    generate_packets(&protocol, &cache)
}

/// Generate packets for the protocol.
fn generate_packets(protocol: &VersionProtocol, _cache: &Path) -> anyhow::Result<()> {
    let output_path = PathBuf::from(file!()).parent().unwrap().join("packets.rs.output");
    let mut output = File::create(&output_path)?;

    for (_name, state) in protocol.packets.iter() {
        for (name, proto) in state.clientbound.iter() {
            if let Some(name) = name.strip_prefix("packet_") {
                generate_struct(name, proto, &mut output)?;
            }
        }
    }

    Ok(())
}

/// Generate a struct name.
fn create_struct_name(name: &str) -> String {
    name.split('/').last().unwrap().to_case(Case::Pascal)
}

/// Generate a packet.
fn generate_struct(name: &str, proto: &ProtocolType, output: &mut File) -> anyhow::Result<String> {
    let struct_name = create_struct_name(name);
    println!("Generating \"{struct_name}\"...");

    match proto {
        // Already exist, just return the name
        ProtocolType::Named(name) => Ok(create_struct_name(name)),
        // Generate the struct and return the name
        ProtocolType::Inline(_, args) => match args {
            // Handle array types
            ProtocolTypeArgs::Array(array_args) => match array_args {
                ArrayArgs::CountField { count_field, kind } => {
                    Ok(format!("[{}; {count_field}]", generate_struct(name, kind, output)?))
                }
                ArrayArgs::Count { kind, .. } => {
                    Ok(format!("Vec<{}>", generate_struct(name, kind, output)?))
                }
            },
            // Handle bitfield types
            ProtocolTypeArgs::Bitfield(..) => Ok(String::from("Bitfield")),
            // Handle buffer types
            ProtocolTypeArgs::Buffer(buffer_args) => match buffer_args {
                BufferArgs::Count(count) => Ok(format!("[u8; {count}]")),
                BufferArgs::CountType(..) => Ok(String::from("Vec<u8>")),
            },
            // Handle container types
            ProtocolTypeArgs::Container(args) => {
                let mut named = Punctuated::new();

                for (index, ContainerArg { name: arg_name, kind }) in args.iter().enumerate() {
                    let arg_name =
                        arg_name.as_ref().map_or(format!("arg{index}"), ToString::to_string);
                    let arg_type = generate_struct(&format!("{name}{arg_name}"), kind, output)?;

                    println!("  {arg_name}: {arg_type}");
                    named.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: Some(Ident::new(&arg_name, Span::call_site())),
                        colon_token: Some(syn::token::Colon::default()),
                        ty: syn::parse_str(&arg_type).unwrap(),
                    });
                }

                let item_struct = ItemStruct {
                    attrs: Vec::new(),
                    vis: Visibility::Inherited,
                    struct_token: syn::token::Struct::default(),
                    ident: Ident::new(&struct_name, Span::call_site()),
                    generics: Generics::default(),
                    semi_token: None,
                    fields: Fields::Named(FieldsNamed {
                        brace_token: syn::token::Brace::default(),
                        named,
                    }),
                };

                let struct_output = prettyplease::unparse(&syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: vec![syn::Item::Struct(item_struct)],
                });
                output.write_all(struct_output.as_bytes())?;

                Ok(struct_name)
            }
            // Handle mapper types
            ProtocolTypeArgs::Mapper(mapper_args) => {
                // TODO: Generate Enum
                Ok(format!("Enum<{}>", generate_struct(name, &mapper_args.kind, output)?))
            }
            // Handle option types
            ProtocolTypeArgs::Option(protocol_type) => {
                Ok(format!("Option<{}>", generate_struct(name, protocol_type, output)?))
            }
            // Handle switch types
            ProtocolTypeArgs::Switch(switch_args) => {
                // TODO: Generate Enum
                Ok(format!("Enum<{}>", create_struct_name(&switch_args.compare_to),))
            }
            // ProtocolTypeArgs::EntityMetadata(metadata_args) => todo!(),
            // ProtocolTypeArgs::PString(buffer_args) => todo!(),
            // ProtocolTypeArgs::TopBitSetTerminatedArray(array_args) => todo!(),
            _ => Ok(String::from("Unsupported")),
        },
    }
}

async fn target_dir() -> PathBuf {
    let mut cache = PathBuf::from(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    while !cache.join("target").exists() {
        assert!(!cache.to_string_lossy().is_empty(), "Couldn't find target directory");
        cache = cache.parent().unwrap().to_path_buf();
    }

    cache.push("target");
    cache.push("froglight-generate");
    tokio::fs::create_dir_all(&cache).await.unwrap();

    cache
}
