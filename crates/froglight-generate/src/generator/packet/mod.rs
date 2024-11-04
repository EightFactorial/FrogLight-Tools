use convert_case::{Case, Casing};
use froglight_parse::file::protocol::{
    ArrayArgs, BitfieldArg, BufferArgs, ContainerArg, EntityMetadataArgs, MapperArgs,
    ProtocolPackets, ProtocolType, ProtocolTypeArgs, SwitchArgs, TopBitSetTerminatedArrayArgs,
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
            // Create a new file for each state
            let mut file = FileWrapper::default();

            // Generate the packet types
            for (packet_name, packet_type) in state_packets
                .clientbound
                .iter()
                .chain(state_packets.serverbound.iter())
                .filter(|(name, _)| name.starts_with("packet_"))
            {
                Self::generate_type(packet_name, packet_type, &mut file)?;
            }

            // Write the file to disk
            let path = std::path::PathBuf::from(file!());
            let path = path.parent().unwrap().join(format!("packets.{state_name}.rs"));

            let unparsed = prettyplease::unparse(&file.into_inner());
            tokio::fs::write(path, unparsed).await?;
        }

        Ok(())
    }

    /// Return the type of a [`ProtocolType`],
    /// generating the type if necessary.
    fn generate_type(
        name: &str,
        proto: &ProtocolType,
        file: &mut FileWrapper,
    ) -> anyhow::Result<Option<String>> {
        match proto {
            ProtocolType::Named(string) => Ok(Some(Self::format_type(string).to_string())),
            ProtocolType::Inline(_, type_args) => Self::generate_args(name, type_args, file),
        }
    }

    fn generate_args(
        name: &str,
        type_args: &ProtocolTypeArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<Option<String>> {
        match type_args {
            ProtocolTypeArgs::Array(array_args) => Self::handle_array(name, array_args, file),
            ProtocolTypeArgs::Bitfield(bitfield_args) => {
                Self::handle_bitfield(name, bitfield_args, file).map(Some)
            }
            ProtocolTypeArgs::Buffer(buffer_args) => {
                Self::handle_buffer(name, buffer_args, file).map(Some)
            }
            ProtocolTypeArgs::Container(container_args) => {
                Self::handle_container(name, container_args, file).map(Some)
            }
            ProtocolTypeArgs::EntityMetadata(metadata_args) => {
                Self::handle_entity_metadata(name, metadata_args, file).map(Some)
            }
            ProtocolTypeArgs::Mapper(mapper_args) => {
                Self::handle_mapper(name, mapper_args, file).map(Some)
            }
            ProtocolTypeArgs::Option(option_type) => {
                Self::handle_option(name, option_type, file).map(Some)
            }
            ProtocolTypeArgs::PString(buffer_args) => {
                Self::handle_pstring(name, buffer_args, file).map(Some)
            }
            ProtocolTypeArgs::Switch(switch_args) => {
                Self::handle_switch(name, switch_args, file).map(Some)
            }
            ProtocolTypeArgs::TopBitSetTerminatedArray(bitset_array_args) => {
                Self::handle_bitset_array(name, bitset_array_args, file).map(Some)
            }
        }
    }
}

#[allow(unused_variables, dead_code)]
#[allow(clippy::unnecessary_wraps)]
impl PacketGenerator {
    fn handle_array(
        name: &str,
        args: &ArrayArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<Option<String>> {
        match args {
            // Create an array with a size determined by `count_type`,
            // which is always a `varint`.
            ArrayArgs::Count { count_type, kind } => {
                assert_eq!(count_type, "varint", "ArrayArgs::Count type must be varint");
                Self::generate_type(&format!("{name}Item"), kind, file)
                    .map(|ty| ty.map(|ty| format!("Vec<{ty}>")))
            }
            // Create an array with a size determined by a field,
            // need to figure out how to handle this.
            ArrayArgs::CountField { count_field, kind } => {
                if let Some(field_type) = file.resolve_field_type(count_field) {
                    if field_type == "varint" {
                        // TODO: Create an enum?
                        return Ok(None);
                    } else if let Some(item) = file.get_struct_mut(&field_type.to_string()) {
                        // TODO: Do something with the struct?
                        return Ok(None);
                    }
                }

                // anyhow::bail!(
                //     "ArrayArgs::CountField unknown field: {} -> {count_field}",
                //     file.last_ident()
                // );
                Ok(None)
            }
        }
    }

    fn handle_bitfield(
        name: &str,
        args: &[BitfieldArg],
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    fn handle_buffer(
        name: &str,
        args: &BufferArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    fn handle_container(
        name: &str,
        args: &[ContainerArg],
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        let struct_name = Self::format_item_name(name);

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
            if let Some(arg_type) = Self::generate_type(&arg_name, kind, file)? {
                file.push_field(&struct_name, &arg_name, &arg_type);
            }
        }

        Ok(struct_name)
    }

    fn handle_entity_metadata(
        name: &str,
        args: &EntityMetadataArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    fn handle_mapper(
        name: &str,
        args: &MapperArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    /// Wrap the inner type in an [`Option`]
    fn handle_option(
        name: &str,
        opt: &ProtocolType,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        match Self::generate_type(name, opt, file) {
            Ok(Some(opt)) => Ok(format!("Option<{opt}>")),
            Ok(None) => Ok(String::from("Option<Unsupported>")),
            Err(err) => Err(err),
        }
    }

    /// This is always a [`String`]
    #[expect(clippy::unnecessary_wraps)]
    fn handle_pstring(
        _name: &str,
        _args: &BufferArgs,
        _file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("String"))
    }

    fn handle_switch(
        name: &str,
        args: &SwitchArgs,
        file: &mut FileWrapper,
    ) -> anyhow::Result<String> {
        Ok(String::from("Unsupported"))
    }

    fn handle_bitset_array(
        name: &str,
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
