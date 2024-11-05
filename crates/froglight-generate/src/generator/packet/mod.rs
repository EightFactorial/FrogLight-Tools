use froglight_parse::file::protocol::{
    ArrayArgs, ArrayWithLengthOffsetArgs, BitfieldArg, BufferArgs, ContainerArg,
    EntityMetadataArgs, MapperArgs, ProtocolPackets, ProtocolStatePackets, ProtocolType,
    ProtocolTypeArgs, SwitchArgs, TopBitSetTerminatedArrayArgs,
};
use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use crate::{cli::CliArgs, datamap::DataMap};

mod result;
use result::Result;

mod state;
use state::State;

mod wrapper;
use wrapper::File;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

impl PacketGenerator {
    /// Generate packets from the given [`DataMap`].
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
        proto_state: &str,
        proto_direction: &str,
        packets: &ProtocolStatePackets,
    ) -> anyhow::Result<()> {
        // Create a new file
        let mut file = File::default();

        // Create a state, only needed for calling `generate_type`
        let root = Ident::new("_", Span::call_site());
        let state = State::new(&root, &root);

        // Generate types for each packet
        for (packet_name, packet_type) in
            packets.iter().filter(|(name, _)| name.starts_with("packet_"))
        {
            let packet_ident = Ident::new(packet_name, Span::call_site());
            if let Result::Err(err) =
                Self::generate_type(state.with_item(&packet_ident), packet_type, &mut file)
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
        let path =
            path.parent().unwrap().join(format!("packets.{proto_state}.{proto_direction}.rs"));

        let unparsed = prettyplease::unparse(&file);
        tokio::fs::write(path, unparsed).await?;
        Ok(())
    }
}

#[allow(unused_variables, dead_code)]
#[allow(clippy::unnecessary_wraps)]
impl PacketGenerator {
    /// Return the type of a [`ProtocolType`],
    /// generating the type if necessary.
    ///
    /// # Note
    /// This may pass back attributes for the type.
    #[must_use]
    fn generate_type(state: State<'_, '_>, proto: &ProtocolType, file: &mut File) -> Result {
        match proto {
            ProtocolType::Named(native) => match native.as_str() {
                "void" => Result::Void,
                other => Result::kind_str(other),
            },
            ProtocolType::Inline(_, type_args) => Self::generate_args(state, type_args, file),
        }
    }

    /// Generate the type specified by the [`ProtocolTypeArgs`].
    ///
    /// # Note
    /// This may pass back attributes for the type.
    #[must_use]
    fn generate_args(state: State<'_, '_>, args: &ProtocolTypeArgs, file: &mut File) -> Result {
        match args {
            ProtocolTypeArgs::Array(args) => Self::handle_array(state, args, file),
            ProtocolTypeArgs::ArrayWithLengthOffset(args) => Self::handle_offset(state, args, file),
            ProtocolTypeArgs::Bitfield(args) => Self::handle_bitfield(state, args, file),
            ProtocolTypeArgs::Buffer(args) => Self::handle_buffer(state, args, file),
            ProtocolTypeArgs::Container(args) => Self::handle_container(state, args, file),
            ProtocolTypeArgs::EntityMetadata(args) => Self::handle_metadata(state, args, file),
            ProtocolTypeArgs::Mapper(args) => Self::handle_mapper(state, args, file),
            ProtocolTypeArgs::Option(option) => Self::handle_option(state, option, file),
            ProtocolTypeArgs::PString(args) => Self::handle_pstring(state, args, file),
            ProtocolTypeArgs::Switch(args) => Self::handle_switch(state, args, file),
            ProtocolTypeArgs::TopBitSetTerminatedArray(args) => {
                Self::handle_bitset(state, args, file)
            }
        }
    }

    /// Handle [`ProtocolTypeArgs::Array`] [`ArrayArgs`]
    #[must_use]
    fn handle_array(state: State<'_, '_>, args: &ArrayArgs, file: &mut File) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::ArrayWithLengthOffset`]
    /// [`ArrayWithLengthOffsetArgs`]
    #[must_use]
    fn handle_offset(
        state: State<'_, '_>,
        args: &ArrayWithLengthOffsetArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Bitfield`] [`BitfieldArg`]
    #[must_use]
    fn handle_bitfield(state: State<'_, '_>, args: &[BitfieldArg], file: &mut File) -> Result {
        // Create the bitfield struct
        let bitfield = state.combined();
        file.create_struct(&bitfield);

        let state = state.with_item(&bitfield);
        for arg in args {
            // Format the field name
            let field_name = Self::format_field_name(&arg.name);

            // Get the field state
            let field_ident = Ident::new(field_name, Span::call_site());
            let field_state = state.with_field(&field_ident);

            // Push the field and attributes
            let size = arg.size;
            file.push_struct_field_str(field_state, "bool")?;
            file.push_field_attr_tokens(field_state, quote! { #[frog(size = #size)] });
        }

        Result::kind_string(&bitfield)
    }

    /// Handle [`ProtocolTypeArgs::Buffer`] [`BufferArgs`]
    #[must_use]
    fn handle_buffer(state: State<'_, '_>, args: &BufferArgs, file: &mut File) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Container`] [`ContainerArg`]
    #[must_use]
    fn handle_container(state: State<'_, '_>, args: &[ContainerArg], file: &mut File) -> Result {
        // Create the container struct
        let container = state.combined();
        file.create_struct(&container);

        let state = state.with_item(&container);
        for (index, arg) in args.iter().enumerate() {
            // Format the field name or create one
            let field_name = if let Some(field_name) = arg.name.as_ref() {
                Self::format_field_name(field_name).to_string()
            } else {
                format!("field_{index}")
            };

            // Get the field state
            let field_ident = Ident::new(&field_name, Span::call_site());
            let field_state = state.with_field(&field_ident);

            // If a type is returned, push it and any attributes
            if let Result::Item { kind, attrs } = Self::generate_type(field_state, &arg.kind, file)?
            {
                file.push_struct_field_type(field_state, kind)?;
                file.push_field_attrs(field_state, attrs);
            }
        }

        Result::kind_string(&container)
    }

    /// Handle [`ProtocolTypeArgs::EntityMetadata`] [`EntityMetadataArgs`]
    #[must_use]
    fn handle_metadata(state: State<'_, '_>, args: &EntityMetadataArgs, file: &mut File) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Mapper`] [`MapperArgs`]
    #[must_use]
    fn handle_mapper(state: State<'_, '_>, args: &MapperArgs, file: &mut File) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Option`] [`ProtocolType`]
    ///
    /// Always maps to `Option<T>`
    #[must_use]
    fn handle_option(state: State<'_, '_>, opt: &ProtocolType, file: &mut File) -> Result {
        Self::generate_type(state, opt, file).map(|ty| format!("Option<{ty}>"))
    }

    /// Handle [`ProtocolTypeArgs::PString`] [`BufferArgs`]
    ///
    /// Always returns `String`
    #[must_use]
    fn handle_pstring(_state: State<'_, '_>, _args: &BufferArgs, _file: &mut File) -> Result {
        Result::kind_str("String")
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    #[must_use]
    fn handle_switch(state: State<'_, '_>, args: &SwitchArgs, file: &mut File) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::TopBitSetTerminatedArray`]
    /// [`TopBitSetTerminatedArrayArgs`]
    #[must_use]
    fn handle_bitset(
        state: State<'_, '_>,
        args: &TopBitSetTerminatedArrayArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }
}

impl PacketGenerator {
    /// Format a field name to prevent conflicts with Rust keywords.
    #[must_use]
    fn format_field_name(field_name: &str) -> &str {
        match field_name {
            "fn" => "fn_",
            "gen" => "gen_",
            "in" => "in_",
            "match" => "match_",
            "mod" => "mod_",
            "mut" => "mut_",
            "ref" => "ref_",
            "type" => "type_",
            other => other,
        }
    }
}
