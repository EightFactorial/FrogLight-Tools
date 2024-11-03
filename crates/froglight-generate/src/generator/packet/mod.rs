use std::path::PathBuf;

use compact_str::CompactString;
use convert_case::{Case, Casing};
use froglight_parse::file::protocol::{
    ArrayArgs, BitfieldArg, BufferArgs, ProtocolPackets, ProtocolType, ProtocolTypeArgs,
};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Field, FieldMutability, Fields, FieldsNamed, File, Generics, Ident,
    Item, ItemEnum, ItemStruct, Token, Variant, Visibility,
};

use crate::{cli::CliArgs, datamap::DataMap};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

impl PacketGenerator {
    pub async fn generate(datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> {
        let dataset = datamap.version_data.iter().next().unwrap().1;
        Self::generate_packets(&dataset.proto.packets).await
    }

    async fn generate_packets(packets: &ProtocolPackets) -> anyhow::Result<()> {
        for (name, state) in packets.iter() {
            let mut file = File { shebang: None, attrs: Vec::new(), items: Vec::new() };

            // Generate structs and enums
            for (name, proto) in state
                .clientbound
                .iter()
                .chain(state.serverbound.iter())
                .filter(|(n, _)| n.starts_with("packet_"))
            {
                Self::generate_type(name, proto, &mut file)?;
            }

            // Write the file to disk
            let file_output = prettyplease::unparse(&file);
            let file_path =
                PathBuf::from(file!()).parent().unwrap().join(format!("packets.{name}.rs"));
            tokio::fs::write(file_path, file_output).await?;
        }

        Ok(())
    }

    fn generate_type(name: &str, proto: &ProtocolType, file: &mut File) -> anyhow::Result<String> {
        match proto {
            // Return the named type
            ProtocolType::Named(string) => Ok(string.to_string()),
            ProtocolType::Inline(_, type_args) => match type_args {
                // Generate the struct
                ProtocolTypeArgs::Array(..)
                | ProtocolTypeArgs::Buffer(..)
                | ProtocolTypeArgs::Container(..) => Self::generate_struct(name, proto, file),
                // Generate the bitfield
                ProtocolTypeArgs::Bitfield(args) => Self::generate_bitfield(name, args, file),
                // Wrap the generated struct in an Option
                ProtocolTypeArgs::Option(proto) => {
                    Self::generate_type(name, proto, file).map(|ty| format!("Option<{ty}>"))
                }
                // Generate the enum
                ProtocolTypeArgs::Mapper(..) | ProtocolTypeArgs::Switch(..) => {
                    Self::generate_enum(name, proto, file)
                }
                // TODO: Implement these
                // ProtocolTypeArgs::EntityMetadata(entity_metadata_args) => todo!(),
                // ProtocolTypeArgs::PString(buffer_args) => todo!(),
                // ProtocolTypeArgs::TopBitSetTerminatedArray(array_args) => todo!(),
                _ => Ok(String::from("Unsupported")),
            },
        }
    }

    fn create_item_name(name: &str) -> String {
        let mut name = name.split('/').last().unwrap();
        if let Some((_, striped)) = name.split_once(':') {
            name = striped;
        }
        name.replace(['.', ':'], "_").to_case(Case::Pascal)
    }

    fn generate_struct(
        name: &str,
        proto: &ProtocolType,
        file: &mut File,
    ) -> anyhow::Result<String> {
        let struct_name = Self::create_item_name(name);
        let ProtocolType::Inline(_, args) = proto else {
            unreachable!("ProtocolType::Named can't be passed to generate_enum");
        };

        match args {
            // Create an array
            ProtocolTypeArgs::Array(array_args) => match array_args {
                // TODO: Remove other field and replace with Vec?
                ArrayArgs::CountField { .. } => Ok(String::from("Unsupported")),
                // Wrap the generated struct in a Vec
                ArrayArgs::Count { count_type, kind } => {
                    if count_type != "varint" {
                        anyhow::bail!(
                            "PacketGenerator: Array has unsupported count type, \"{}\"",
                            count_type
                        );
                    }

                    let array_type =
                        Self::generate_type(&format!("{struct_name}Item"), kind, file)?;
                    Ok(format!("Vec<{array_type}>"))
                }
            },
            // Create a buffer
            ProtocolTypeArgs::Buffer(buffer_args) => match buffer_args {
                // Use the count as the size of the buffer
                BufferArgs::Count(count) => Ok(format!("[u8; {count}]")),
                // This is always a Vec<u8>
                BufferArgs::CountType(count_type) => {
                    if count_type != "varint" {
                        anyhow::bail!(
                            "PacketGenerator: Buffer has unsupported count type, \"{}\"",
                            count_type
                        );
                    }

                    Ok(String::from("Vec<u8>"))
                }
            },
            // Create a new struct
            ProtocolTypeArgs::Container(container_args) => {
                let mut fields = Punctuated::new();
                for (index, arg) in container_args.iter().enumerate() {
                    let arg_name =
                        arg.name.as_ref().map_or(format!("arg{index}"), CompactString::to_string);
                    let arg_type = Self::generate_type(&arg_name, &arg.kind, file)?;

                    fields.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Public(<Token![pub]>::default()),
                        mutability: FieldMutability::None,
                        ident: Some(Ident::new(&arg_name, Span::call_site())),
                        colon_token: None,
                        ty: syn::parse_str(&arg_type).unwrap(),
                    });
                }

                file.items.push(Item::Struct(ItemStruct {
                    attrs: Vec::new(),
                    vis: Visibility::Public(<Token![pub]>::default()),
                    struct_token: <Token![struct]>::default(),
                    ident: Ident::new(&struct_name, Span::call_site()),
                    generics: Generics::default(),
                    semi_token: None,
                    fields: Fields::Named(FieldsNamed {
                        brace_token: syn::token::Brace::default(),
                        named: fields,
                    }),
                }));

                Ok(struct_name)
            }
            _ => unreachable!("Only Array, Buffer, and Container are supported for structs"),
        }
    }

    #[expect(clippy::unnecessary_wraps)]
    fn generate_bitfield(
        _name: &str,
        _args: &[BitfieldArg],
        _file: &mut File,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    fn generate_enum(name: &str, proto: &ProtocolType, file: &mut File) -> anyhow::Result<String> {
        let enum_name = Self::create_item_name(name);
        let ProtocolType::Inline(_, args) = proto else {
            unreachable!("ProtocolType::Named can't be passed to generate_enum");
        };

        let mut variants = Punctuated::new();
        match args {
            // Read a `varint` and map it to an enum variant
            ProtocolTypeArgs::Mapper(mapper_args) => {
                // Ensure the enum id is mapped to a varint
                if *mapper_args.kind != ProtocolType::Named(CompactString::const_new("varint")) {
                    anyhow::bail!(
                        "PacketGenerator: Enum has unsupported type, \"{:?}\"",
                        mapper_args.kind
                    );
                }

                // Sort the variants by id
                let mut mappings = Vec::with_capacity(mapper_args.mappings.len());
                mappings.extend(mapper_args.mappings.iter());
                mappings.sort_by(|(a, _), (b, _)| a.cmp(b));

                for (_, variant) in mappings {
                    let variant = Self::create_item_name(variant);
                    match syn::parse_str::<Variant>(&variant) {
                        Ok(variant) => variants.push(variant),
                        Err(err) => {
                            anyhow::bail!(
                                "PacketGenerator: Failed to parse variant, \"{variant}\": {err}",
                            );
                        }
                    }
                }
            }
            // TODO: Remove other field and replace with an enum?
            ProtocolTypeArgs::Switch(..) => return Ok(String::from("Unsupported")),
            _ => unreachable!("Only Mapper and Switch are supported for enums"),
        }

        file.items.push(Item::Enum(ItemEnum {
            attrs: Vec::new(),
            vis: Visibility::Public(<Token![pub]>::default()),
            enum_token: <Token![enum]>::default(),
            ident: Ident::new(&enum_name, Span::call_site()),
            generics: Generics::default(),
            brace_token: syn::token::Brace::default(),
            variants,
        }));

        Ok(enum_name)
    }
}
