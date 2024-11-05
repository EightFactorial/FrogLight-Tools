use convert_case::{Case, Casing};
use froglight_parse::file::protocol::{
    ArrayArgs, ArrayWithLengthOffsetArgs, BitfieldArg, BufferArgs, ContainerArg,
    EntityMetadataArgs, MapperArgs, ProtocolPackets, ProtocolStatePackets, ProtocolType,
    ProtocolTypeArgs, SwitchArgs, TopBitSetTerminatedArrayArgs,
};

use crate::{cli::CliArgs, datamap::DataMap};

mod wrapper;
use wrapper::FileWrapper;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

impl PacketGenerator {
    pub async fn generate(datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> {
        let dataset = datamap.version_data.iter().next().unwrap().1;
        Self::generate_packets(&dataset.proto.packets).await
    }

    async fn generate_packets(packets: &ProtocolPackets) -> anyhow::Result<()> {
        for (state_name, state_packets) in packets.iter() {
            Self::generate_state_packets(state_name, "toClient", &state_packets.clientbound)
                .await?;
            Self::generate_state_packets(state_name, "toServer", &state_packets.serverbound)
                .await?;
        }
        Ok(())
    }

    async fn generate_state_packets(
        state: &str,
        direction: &str,
        packets: &ProtocolStatePackets,
    ) -> anyhow::Result<()> {
        // Create a new file
        let mut file = FileWrapper::default();
        for (packet_name, packet_type) in
            packets.iter().filter(|(name, _)| name.starts_with("packet_"))
        {
            if let Err(err) = Self::generate_type(packet_name, packet_name, packet_type, &mut file)
            {
                tracing::error!("Error generating type for {packet_name}: {err}");
            }
        }

        // If the file is empty, return early
        let file = file.into_inner();
        if file.items.is_empty() {
            return Ok(());
        }

        // Write the file to disk
        let path = std::path::PathBuf::from(file!());
        let path = path.parent().unwrap().join(format!("packets.{state}.{direction}.rs"));

        let unparsed = prettyplease::unparse(&file);
        tokio::fs::write(path, unparsed).await?;
        Ok(())
    }

    /// Return the type of a [`ProtocolType`],
    /// generating the type if necessary.
    fn generate_type(
        struct_ident: &str,
        field_ident: &str,
        proto: &ProtocolType,
        file: &mut FileWrapper,
    ) -> anyhow::Result<Option<String>> {
        match proto {
            ProtocolType::Named(string) => match string.as_str() {
                "void" => Ok(None),
                other => Ok(Some(Self::format_type(other).to_string())),
            },
            ProtocolType::Inline(_, type_args) => {
                Self::generate_args(struct_ident, field_ident, type_args, file)
            }
        }
    }

    fn generate_args(
        struct_ident: &str,
        field_ident: &str,
        type_args: &ProtocolTypeArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<Option<String>> {
        match type_args {
            ProtocolTypeArgs::Array(array_args) => {
                Self::handle_array(struct_ident, field_ident, array_args, file)
            }
            ProtocolTypeArgs::ArrayWithLengthOffset(array_args) => {
                Self::handle_array_with_length_offset(struct_ident, field_ident, array_args, file)
                    .map(Some)
            }
            ProtocolTypeArgs::Bitfield(bitfield_args) => {
                Self::handle_bitfield(struct_ident, field_ident, bitfield_args, file).map(Some)
            }
            ProtocolTypeArgs::Buffer(buffer_args) => {
                Self::handle_buffer(struct_ident, field_ident, buffer_args, file).map(Some)
            }
            ProtocolTypeArgs::Container(container_args) => {
                Self::handle_container(struct_ident, field_ident, container_args, file).map(Some)
            }
            ProtocolTypeArgs::EntityMetadata(metadata_args) => {
                Self::handle_entity_metadata(struct_ident, field_ident, metadata_args, file)
                    .map(Some)
            }
            ProtocolTypeArgs::Mapper(mapper_args) => {
                Self::handle_mapper(struct_ident, field_ident, mapper_args, file).map(Some)
            }
            ProtocolTypeArgs::Option(option_type) => {
                Self::handle_option(struct_ident, field_ident, option_type, file).map(Some)
            }
            ProtocolTypeArgs::PString(buffer_args) => {
                Self::handle_pstring(struct_ident, field_ident, buffer_args, file).map(Some)
            }
            ProtocolTypeArgs::Switch(switch_args) => {
                Self::handle_switch(struct_ident, field_ident, switch_args, file)?;
                Ok(None)
            }
            ProtocolTypeArgs::TopBitSetTerminatedArray(bitset_array_args) => {
                Self::handle_bitset_array(struct_ident, field_ident, bitset_array_args, file)
                    .map(Some)
            }
        }
    }
}

#[allow(unused_variables, dead_code)]
#[allow(clippy::unnecessary_wraps)]
impl PacketGenerator {
    fn handle_array(
        struct_ident: &str,
        field_ident: &str,
        args: &ArrayArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<Option<String>> {
        match args {
            // Create an array with a size determined by `count_type`,
            // which is always a `varint`.
            ArrayArgs::Count { count_type, kind } => {
                assert_eq!(count_type, "varint", "ArrayArgs::Count type must be varint");
                Self::generate_type(struct_ident, &format!("{field_ident}Item"), kind, file)
                    .map(|ty| ty.map(|ty| format!("Vec<{ty}>")))
            }
            // Create an array with a size determined by a field,
            // need to figure out how to handle this.
            ArrayArgs::CountField { count_field, kind } => {
                let field_type = file.resolve_field_type(struct_ident, count_field)?;
                assert!(field_type == "VarInt", "ArrayArgs::CountField type must be VarInt");

                if let Some(count_type) =
                    Self::generate_type(struct_ident, &format!("{field_ident}Item"), kind, file)?
                {
                    Ok(Some(format!("Vec<{count_type}>")))
                } else {
                    anyhow::bail!("ArrayArgs::CountField type must be a valid type");
                }
            }
        }
    }

    fn handle_array_with_length_offset(
        struct_ident: &str,
        field_ident: &str,
        args: &ArrayWithLengthOffsetArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    /// Create a struct for the bitfield
    fn handle_bitfield(
        struct_ident: &str,
        field_ident: &str,
        args: &[BitfieldArg],
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        let bitfield_name = Self::format_item_name(field_ident) + "Bitfield";

        // Create a new struct for the bitfield
        // TODO: Add field attributes
        file.push_struct(&bitfield_name);
        for arg in args {
            let field_name = Self::format_field_name(&arg.name);
            file.push_field(&bitfield_name, &field_name, "bool");
        }

        Ok(bitfield_name)
    }

    /// This always returns either `[u8; N]` or `Vec<u8>`
    fn handle_buffer(
        struct_ident: &str,
        field_ident: &str,
        args: &BufferArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        match args {
            BufferArgs::Count(count) => Ok(format!("[u8; {count}]")),
            BufferArgs::CountType(count_type) => {
                assert_eq!(count_type, "varint", "BufferArgs::CountType type must be varint");
                Ok(String::from("Vec<u8>"))
            }
        }
    }

    /// Create a struct for the container
    fn handle_container(
        struct_ident: &str,
        field_ident: &str,
        args: &[ContainerArg],
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        let struct_name = Self::format_item_name(field_ident);

        // Create a new struct for the container
        file.push_struct(&struct_name);
        for (index, ContainerArg { name, kind }) in args.iter().enumerate() {
            // Format the existing field name or generate a new one
            let arg_name = if let Some(name) = name {
                Self::format_field_name(name)
            } else {
                format!("field_{index}")
            };

            // Get the type of the field, generating it if necessary
            if let Some(arg_type) = Self::generate_type(&struct_name, &arg_name, kind, file)? {
                file.push_field(&struct_name, &arg_name, &arg_type);
            }
        }

        Ok(struct_name)
    }

    /// TODO: What is this?
    fn handle_entity_metadata(
        struct_ident: &str,
        field_ident: &str,
        args: &EntityMetadataArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    /// Create an enum for the mapper
    fn handle_mapper(
        struct_ident: &str,
        field_ident: &str,
        args: &MapperArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        let enum_name = struct_ident.to_string() + &Self::format_item_name(field_ident);

        // Create a new enum for the mapper
        file.push_enum(&enum_name);

        // Sort the mappings by corresponding case
        let mut collection: Vec<_> = args.mappings.iter().collect();
        collection
            .sort_by_key(|(case, _)| case.parse::<u32>().expect("Mapper case must be a number"));

        // Add each variant to the enum
        for (case, name) in collection {
            let variant = Self::format_item_name(name);
            file.push_variant(&enum_name, &variant, Some(case));
        }

        Ok(enum_name)
    }

    /// Wrap the inner type in an [`Option`]
    fn handle_option(
        struct_ident: &str,
        field_ident: &str,
        opt: &ProtocolType,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        match Self::generate_type(struct_ident, field_ident, opt, file) {
            Ok(Some(opt)) => Ok(format!("Option<{opt}>")),
            Ok(None) => Ok(String::from("Option<Unsupported>")),
            Err(err) => Err(err),
        }
    }

    /// This is always a [`String`]
    #[expect(clippy::unnecessary_wraps)]
    fn handle_pstring(
        _struct_ident: &str,
        _field_ident: &str,
        _args: &BufferArgs,
        _file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("String"))
    }

    fn handle_switch(
        struct_ident: &str,
        field_ident: &str,
        args: &SwitchArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<()> {
        file.convert_or_modify_enum(struct_ident, args)
    }

    /// TODO: How should this be handled?
    fn handle_bitset_array(
        struct_ident: &str,
        field_ident: &str,
        args: &TopBitSetTerminatedArrayArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }
}

impl PacketGenerator {
    /// Format a field name to be valid in Rust
    fn format_field_name(field: &str) -> String {
        match field {
            "match" => String::from("match_"),
            "mod" => String::from("mod_"),
            "ref" => String::from("ref_"),
            "type" => String::from("type_"),
            other => other.to_case(Case::Snake),
        }
    }

    /// Format type names to match Rust conventions
    fn format_type(ty: &str) -> &str {
        match ty {
            "anonOptionalNbt" => "Option<Nbt>",
            "anonymousNbt" => "Nbt",
            "position" => "Position",
            "restBuffer" => "UnsizedBuffer",
            "string" => "String",
            "UUID" => "Uuid",
            "varint" => "VarInt",
            "vec2f" => "Vec2",
            "vec2f64" => "DVec2",
            "vec3f" => "Vec3",
            "vec3f64" => "DVec3",
            "minecraft_simple_recipe_format" => "CraftingRecipe",
            "minecraft_smelting_format" => "SmeltingRecipe",
            "ingredient" => "RecipeIngredient",
            other => other,
        }
    }

    /// Format item names to match Rust conventions
    fn format_item_name(name: &str) -> String {
        let mut name = name.split('/').last().unwrap();
        if let Some((_, striped)) = name.split_once(':') {
            name = striped;
        }
        name.replace(['.', ':'], "_").to_case(Case::Pascal)
    }
}
