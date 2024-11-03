use std::path::PathBuf;

use compact_str::CompactString;
use convert_case::{Case, Casing};
use froglight_parse::file::protocol::{
    ArrayArgs, BitfieldArg, BufferArgs, ContainerArg, EntityMetadataArgs, MapperArgs,
    ProtocolPackets, ProtocolType, ProtocolTypeArgs, SwitchArgs, TopBitSetTerminatedArrayArgs,
};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, Field, FieldMutability, Fields, FieldsNamed, File, Generics, Ident,
    Item, ItemEnum, ItemStruct, Token, Visibility,
};

use crate::{cli::CliArgs, datamap::DataMap};

mod parent;
use parent::ParentStack;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

#[allow(clippy::unnecessary_wraps)]
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
                let mut parents = ParentStack::new();
                Self::generate_type(name, proto, &mut parents, &mut file)?;
            }

            // Write the file to disk
            let file_output = prettyplease::unparse(&file);
            let file_path =
                PathBuf::from(file!()).parent().unwrap().join(format!("packets.{name}.rs"));
            tokio::fs::write(file_path, file_output).await?;
        }

        Ok(())
    }

    fn generate_type(
        field: &str,
        proto: &ProtocolType,
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<Option<String>> {
        match proto {
            ProtocolType::Named(name) => Ok(Some(name.to_string())),
            ProtocolType::Inline(.., type_args) => {
                Self::generate_type_args(field, type_args, parents, file)
            }
        }
    }

    fn generate_type_args(
        field: &str,
        proto_args: &ProtocolTypeArgs,
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<Option<String>> {
        match proto_args {
            ProtocolTypeArgs::Array(array_args) => {
                Self::handle_array(field, array_args, parents, file)
            }
            ProtocolTypeArgs::Bitfield(bitfield_args) => {
                Self::handle_bitfield(field, bitfield_args, parents, file).map(Some)
            }
            ProtocolTypeArgs::Buffer(buffer_args) => Self::handle_buffer(buffer_args).map(Some),
            ProtocolTypeArgs::Container(container_args) => {
                Self::handle_container(field, container_args, parents, file).map(Some)
            }
            ProtocolTypeArgs::EntityMetadata(entity_metadata_args) => {
                Self::handle_entity_metadata(field, entity_metadata_args, parents, file).map(Some)
            }
            ProtocolTypeArgs::Mapper(mapper_args) => {
                Self::handle_mapper(field, mapper_args, parents, file).map(Some)
            }
            ProtocolTypeArgs::Option(protocol_type) => {
                Self::handle_option(field, protocol_type, parents, file).map(Some)
            }
            ProtocolTypeArgs::PString(buffer_args) => Self::handle_pstring(buffer_args).map(Some),
            ProtocolTypeArgs::Switch(switch_args) => {
                Self::handle_switch(field, switch_args, parents, file)?;
                Ok(None)
            }
            ProtocolTypeArgs::TopBitSetTerminatedArray(bitset_args) => {
                Self::handle_top_bitset_terminated_array(field, bitset_args, parents, file)
                    .map(Some)
            }
        }
    }

    fn handle_container(
        field: &str,
        container_args: &[ContainerArg],
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<String> {
        let struct_name = Self::create_item_name(field);

        // Push a new struct to the parent stack
        parents.push(ItemStruct {
            attrs: Vec::new(),
            vis: Visibility::Public(<Token![pub]>::default()),
            struct_token: <Token![struct]>::default(),
            ident: Ident::new(&struct_name, Span::call_site()),
            generics: Generics::default(),
            fields: Fields::Named(FieldsNamed {
                brace_token: syn::token::Brace::default(),
                named: Punctuated::new(),
            }),
            semi_token: None,
        });

        for (index, container_arg) in container_args.iter().enumerate() {
            let arg_name = container_arg
                .name
                .as_ref()
                .map_or(format!("field_{index}"), CompactString::to_string);

            if let Some(arg_type) =
                Self::generate_type(&arg_name, &container_arg.kind, parents, file)?
            {
                // Add the field to the current struct
                if let Fields::Named(fields) = &mut parents.fields {
                    fields.named.push(Self::create_field(&arg_name, &arg_type)?);
                }
            }
        }

        // Pop the struct from the parent stack and add it to the file
        if let Some(item_struct) = parents.pop() {
            file.items.push(Item::Struct(item_struct));
        }

        Ok(struct_name)
    }

    fn handle_switch(
        _field: &str,
        switch_args: &SwitchArgs,
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<()> {
        let _field = parents.get_field(&switch_args.compare_to, file)?;

        Ok(())
    }

    fn handle_array(
        field: &str,
        array_args: &ArrayArgs,
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<Option<String>> {
        match array_args {
            ArrayArgs::Count { count_type, kind } => {
                if count_type != "varint" {
                    anyhow::bail!("ArrayArgs: Unsupported type \"{count_type}\"");
                }

                Ok(Self::generate_type(field, kind, parents, file)?.map(|ty| format!("Vec<{ty}>")))
            }
            ArrayArgs::CountField { count_field: _, kind: _ } => {
                Ok(Some(String::from("Vec<Unsupported>")))
            }
        }
    }

    fn handle_bitfield(
        field: &str,
        bitfield_args: &[BitfieldArg],
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<String> {
        let bitfield_name = Self::create_item_name(field) + "BitField";

        // Push a new struct to the parent stack
        parents.push(ItemStruct {
            attrs: Vec::new(),
            vis: Visibility::Public(<Token![pub]>::default()),
            struct_token: <Token![struct]>::default(),
            ident: Ident::new(&bitfield_name, Span::call_site()),
            generics: Generics::default(),
            fields: Fields::Named(FieldsNamed {
                brace_token: syn::token::Brace::default(),
                named: Punctuated::new(),
            }),
            semi_token: None,
        });

        // Add the bitfield arguments as fields to the struct
        for bitfield_arg in bitfield_args {
            if let Fields::Named(fields) = &mut parents.fields {
                fields.named.push(Self::create_field(&bitfield_arg.name, "bool")?);
            }
        }

        // Pop the struct from the parent stack and add it to the file
        if let Some(item_struct) = parents.pop() {
            file.items.push(Item::Struct(item_struct));
        }
        Ok(bitfield_name)
    }

    fn handle_buffer(buffer_args: &BufferArgs) -> anyhow::Result<String> {
        match buffer_args {
            BufferArgs::Count(count) => Ok(format!("[u8; {count}]")),
            BufferArgs::CountType(count_type) => {
                if count_type != "varint" {
                    anyhow::bail!("BufferArgs: Unsupported type \"{count_type}\"");
                }

                Ok(String::from("Vec<u8>"))
            }
        }
    }

    fn handle_entity_metadata(
        _field: &str,
        _metadata_args: &EntityMetadataArgs,
        _parents: &mut ParentStack,
        _file: &mut File,
    ) -> anyhow::Result<String> {
        Ok(String::from("EntityMetadata<Unsupported>"))
    }

    fn handle_mapper(
        field: &str,
        mapper_args: &MapperArgs,
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<String> {
        let enum_name = parents.ident.to_string() + &Self::create_item_name(field);
        let Some(mapper_type) = Self::generate_type(field, &mapper_args.kind, parents, file)?
        else {
            anyhow::bail!("MapperArgs: Missing type");
        };

        if mapper_type != "varint" {
            anyhow::bail!("MapperArgs: Unsupported type \"{mapper_type}\"");
        }

        let mut variants = Punctuated::new();
        for (_case, result) in &mapper_args.mappings {
            let variant = Ident::new(&Self::create_item_name(result), Span::call_site());
            variants.push(syn::parse_quote! { #variant
            });
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

    fn handle_option(
        field: &str,
        proto: &ProtocolType,
        parents: &mut ParentStack,
        file: &mut File,
    ) -> anyhow::Result<String> {
        match Self::generate_type(field, proto, parents, file) {
            Ok(Some(ty)) => Ok(format!("Option<{ty}>")),
            Ok(None) => Ok(String::from("Option<Unsupported>")),
            Err(err) => Err(err),
        }
    }

    fn handle_pstring(_buffer_args: &BufferArgs) -> anyhow::Result<String> {
        Ok(String::from("String"))
    }

    fn handle_top_bitset_terminated_array(
        _field: &str,
        _bitset_args: &TopBitSetTerminatedArrayArgs,
        _parents: &mut ParentStack,
        _file: &mut File,
    ) -> anyhow::Result<String> {
        Ok(String::from("BitSetArray<Unsupported>"))
    }

    fn create_item_name(name: &str) -> String {
        let mut name = name.split('/').last().unwrap();
        if let Some((_, striped)) = name.split_once(':') {
            name = striped;
        }
        name.replace(['.', ':'], "_").to_case(Case::Pascal)
    }

    /// Create a [`Field`] from an [`Ident`] and a [`Type`](syn::Type).
    fn create_field(ident: &str, ty: &str) -> anyhow::Result<Field> {
        match syn::parse_str(ty) {
            Err(err) => anyhow::bail!("Failed to parse field \"{ident}: {ty}\": {err}"),
            Ok(ty) => Ok(Field {
                attrs: Vec::new(),
                vis: Visibility::Public(<Token![pub]>::default()),
                mutability: FieldMutability::None,
                ident: Some(Ident::new(ident, Span::call_site())),
                colon_token: Some(<Token![:]>::default()),
                ty,
            }),
        }
    }
}
