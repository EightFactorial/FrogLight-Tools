use froglight_parse::file::protocol::{
    ArrayArgs, ArrayWithLengthOffsetArgs, BitfieldArg, BufferArgs, ContainerArg,
    EntityMetadataArgs, MapperArgs, ProtocolPackets, ProtocolStatePackets, ProtocolType,
    ProtocolTypeArgs, SwitchArgs, TopBitSetTerminatedArrayArgs,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Expr, Ident, Lit, LitInt, Type};

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
                tracing::error!("Error generating type for \"{packet_name}\": {err}");
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
                Self::handle_bitset_array(state, args, file)
            }
        }
    }

    /// Handle [`ProtocolTypeArgs::Array`] [`ArrayArgs`]
    #[must_use]
    fn handle_array(state: State<'_, '_>, args: &ArrayArgs, file: &mut File) -> Result {
        match args {
            ArrayArgs::CountField { count_field, kind } => {
                let count_field = Self::format_field_name(count_field);
                let count_ident = Ident::new(count_field, Span::call_site());

                let count_state = state.with_field(&count_ident);

                Self::generate_type(count_state, kind, file)
                    .map(|ty| format!("Vec<{ty}>"))
                    .with_attr_tokens(quote! {
                        #[frog(length = #count_ident)]
                    })
            }
            ArrayArgs::Count { count_type, kind } => {
                assert_eq!(count_type, "varint", "ArrayArgs: Unsupported type \"{count_type}\"");
                Self::generate_type(state, kind, file).map(|ty| format!("Vec<{ty}>"))
            }
        }
    }

    /// Handle [`ProtocolTypeArgs::ArrayWithLengthOffset`]
    /// [`ArrayWithLengthOffsetArgs`]
    #[must_use]
    fn handle_offset(
        state: State<'_, '_>,
        args: &ArrayWithLengthOffsetArgs,
        file: &mut File,
    ) -> Result {
        let offset = LitInt::new(&args.length_offset.to_string(), Span::call_site());
        Self::handle_array(state, &args.array, file)
            .with_attr_tokens(quote! { #[frog(offset = #offset)] })
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

            // Get the argument type
            let arg_type = match arg.size {
                1 => "bool",
                2..=8 => "u8",
                9..=16 => "u16",
                17..=32 => "u32",
                33..=64 => "u64",
                _ => panic!("BitfieldArg: Unsupported size \"{}\"", arg.size),
            };

            // Push the field and attributes
            file.push_struct_field_str(field_state, arg_type)?;
            let field_size = LitInt::new(&arg.size.to_string(), Span::call_site());
            file.push_field_attr_tokens(field_state, quote! { #[frog(field_size = #field_size)] });
        }

        Result::kind_string(&bitfield)
    }

    /// Handle [`ProtocolTypeArgs::Buffer`] [`BufferArgs`]
    #[must_use]
    fn handle_buffer(state: State<'_, '_>, args: &BufferArgs, file: &mut File) -> Result {
        match args {
            BufferArgs::Count(count) => Result::kind_str(format!("[u8; {count}]")),
            BufferArgs::CountType(count_type) => {
                assert_eq!(count_type, "varint", "BufferArgs: Unsupported type \"{count_type}\"");
                Result::kind_str("Vec<u8>")
            }
        }
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
        tracing::error!("MetadataArgs: Unsupported \"{}.{}\"", state.item, state.field);
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Mapper`] [`MapperArgs`]
    #[must_use]
    fn handle_mapper(state: State<'_, '_>, args: &MapperArgs, file: &mut File) -> Result {
        // Create the mapper enum
        let mapper = state.combined();
        file.create_enum(&mapper);

        let mut collection: Vec<_> = args.mappings.iter().collect();
        collection.sort_by_key(|(key, _)| key.parse::<isize>().unwrap());

        let state = state.with_item(&mapper);
        for (input, output) in collection {
            // Create the variant descriptor
            let descriptor = LitInt::new(input, Span::call_site());

            // Format the variant name
            let variant_name = Self::format_variant_name(output);
            let variant_ident = if variant_name.starts_with(char::is_numeric) {
                Ident::new(&format!("when_{variant_name}"), Span::call_site())
            } else {
                Ident::new(&variant_name, Span::call_site())
            };

            // Push the variant
            file.push_enum_variant_tokens(state, quote!(#variant_ident = #descriptor))?;
        }

        Result::kind_string(&mapper)
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
    /// Always returns [`String`]
    #[must_use]
    fn handle_pstring(_state: State<'_, '_>, _args: &BufferArgs, _file: &mut File) -> Result {
        Result::kind_str("String")
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    #[must_use]
    fn handle_switch(state: State<'_, '_>, args: &SwitchArgs, file: &mut File) -> Result {
        // Warn about unsupported comparison fields, but continue
        if args.compare_to.contains('/') {
            tracing::warn!(
                "SwitchArgs: Unsupported comparison field \"{}.{}\": \"{}\"",
                state.item,
                state.field,
                args.compare_to
            );
            return Result::unsupported();
        }

        // Get the field to compare to
        let compared_field = Self::format_field_name(&args.compare_to);
        let compared_ident = Ident::new(compared_field, Span::call_site());
        let Some(struct_field) = file.get_struct_field(state.with_field(&compared_ident)) else {
            return Result::Err(anyhow::anyhow!(
                "SwitchArgs: Field \"{}.{compared_field}\" not found ",
                state.item,
            ));
        };

        if let Type::Path(type_path) = &struct_field.ty {
            let last = type_path.path.segments.last().unwrap();
            let field_type = last.ident.to_string();

            // TODO: Collapse `Void = 0, Data = N` into `Option<T>`
            // TODO: Support matching on `Option`
            // TODO: Support matching on enum types
            // Pick the generation function by the field type
            match field_type.as_str() {
                "bool" => Self::handle_switch_bool(state, args, file),
                "varint" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                    Self::handle_switch_integer(state, args, file)
                }
                field_enum if field_enum.ends_with('_') => {
                    Self::handle_switch_enum(state, args, file)
                }
                unknown => {
                    Result::Err(anyhow::anyhow!("SwitchArgs: Unsupported type \"{unknown}\"",))
                }
            }
            .with_attr_tokens(quote! { #[frog(match_on = #compared_ident)] })
        } else {
            Result::Err(anyhow::anyhow!(
                "SwitchArgs: Unsupported type \"{}.{compared_field}\"",
                state.item
            ))
        }
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    /// with a boolean comparison
    #[must_use]
    fn handle_switch_bool(state: State<'_, '_>, args: &SwitchArgs, file: &mut File) -> Result {
        // Create the switch enum
        let switch = state.combined();
        file.create_enum(&switch);

        let state = state.with_item(&switch);

        let default_type = args.default.as_ref().map(|ty| Self::generate_type(state, ty, file));

        // Handle the `false` case
        {
            let false_variant = Ident::new("when_false", Span::call_site());
            let false_case =
                args.fields.iter().find_map(|(input, output)| (input == "false").then_some(output));

            let false_type = false_case.map(|ty| Self::generate_type(state, ty, file));
            let mut false_tokens = match &false_type {
                // Use the returned type
                Some(Result::Item { kind, attrs }) => quote!(#false_variant(#(#attrs)* #kind) = 0),
                // Return the error
                Some(Result::Err(err)) => return false_type.unwrap(),
                // Create a Unit variant
                _ => quote!(#false_variant = 0),
            };

            if false_type == default_type {
                false_tokens = quote!(#[frog(default)] #false_tokens);
            }
            file.push_enum_variant_tokens(state, false_tokens)?;
        }

        // Handle the `true` case
        {
            let true_variant = Ident::new("when_true", Span::call_site());
            let true_case =
                args.fields.iter().find_map(|(input, output)| (input == "true").then_some(output));

            let true_type = true_case.map(|ty| Self::generate_type(state, ty, file));
            let mut true_tokens = match &true_type {
                // Use the returned type
                Some(Result::Item { kind, attrs }) => quote!(#true_variant(#(#attrs)* #kind) = 1),
                // Return the error
                Some(Result::Err(err)) => return true_type.unwrap(),
                // Create a Unit variant
                _ => quote!(#true_variant = 1),
            };

            if true_type == default_type {
                true_tokens = quote!(#[frog(default)] #true_tokens);
            }
            file.push_enum_variant_tokens(state, true_tokens)?;
        }

        // Handle the `default` case, if it exists
        if let Some(default_type) = args.default.as_ref() {
            let default = Ident::new("default", Span::call_site());
            if let Result::Item { kind, attrs } = Self::generate_type(state, default_type, file)? {
                file.push_enum_variant_tokens(state, quote!(#default(varint, #(#attrs)* #kind)))?;
            } else {
                file.push_enum_variant_tokens(state, quote!(#default(varint)))?;
            }
        }

        Result::kind_string(state.item)
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    /// with an integer comparison
    #[must_use]
    fn handle_switch_integer(state: State<'_, '_>, args: &SwitchArgs, file: &mut File) -> Result {
        // Create the switch enum
        let switch = state.combined();
        file.create_enum(&switch);

        let state = state.with_item(&switch);

        // Sort the fields by key
        let mut collection: Vec<_> = args.fields.iter().collect();
        collection.sort_by_key(|(key, _)| key.parse::<isize>().unwrap());

        let default_type = args.default.as_ref().map(|ty| Self::generate_type(state, ty, file));

        // Handle each case
        for (input, output) in &collection {
            // Create the variant descriptor
            let descriptor = LitInt::new(input, Span::call_site());

            // Format the variant name
            let variant_name = Self::format_variant_name(input);
            let variant_ident = if variant_name.starts_with(char::is_numeric) {
                Ident::new(&format!("when_{variant_name}"), Span::call_site())
            } else {
                Ident::new(&variant_name, Span::call_site())
            };

            // Generate the type and push the variant
            let variant_type = Self::generate_type(state, output, file)?;
            if let Result::Item { kind, attrs } = &variant_type {
                file.push_enum_variant_tokens(
                    state,
                    quote!(#variant_ident(#(#attrs)* #kind) = #descriptor),
                )?;
            } else {
                file.push_enum_variant_tokens(state, quote!(#variant_ident = #descriptor))?;
            };
        }

        // Handle the `default` case, if it exists
        if let Some(default_type) = args.default.as_ref() {
            let default = Ident::new("default", Span::call_site());
            if let Result::Item { kind, attrs } = Self::generate_type(state, default_type, file)? {
                file.push_enum_variant_tokens(
                    state,
                    quote! { #[frog(default)] #default(varint, #(#attrs)* #kind) },
                )?;
            } else {
                file.push_enum_variant_tokens(state, quote! { #[frog(default)] #default(varint) })?;
            }
        }

        Result::kind_string(&switch)
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    /// with an enum comparison
    #[must_use]
    fn handle_switch_enum(state: State<'_, '_>, args: &SwitchArgs, file: &mut File) -> Result {
        // Create the switch enum
        let switch = state.combined();
        file.create_enum(&switch);

        let compared_field = Self::format_field_name(&args.compare_to);
        let compared_ident = Ident::new(compared_field, Span::call_site());
        let compared_state = state.with_field(&compared_ident);

        let enum_type = file.get_struct_field_type(compared_state)?.clone();
        let enum_state = state.with_item(&enum_type);

        // Handle each case
        let switch_state = state.with_item(&switch);
        for (input, output) in &args.fields {
            // Format the variant name
            let variant_name = Self::format_variant_name(input);
            let variant_ident = if variant_name.starts_with(char::is_numeric) {
                Ident::new(&format!("when_{variant_name}"), Span::call_site())
            } else {
                Ident::new(&variant_name, Span::call_site())
            };

            // Get the variant descriptor
            if let Some(enum_variant) = file.get_enum_variant(enum_state.with_field(&variant_ident))
            {
                let mut descriptor_tokens = TokenStream::new();
                if let Some(descriptor) = enum_variant.discriminant.as_ref() {
                    descriptor_tokens.extend(descriptor.0.to_token_stream());
                    descriptor_tokens.extend(descriptor.1.to_token_stream());
                }

                // Generate the type and push the variant
                let variant_type = Self::generate_type(state, output, file)?;
                if let Result::Item { kind, attrs } = &variant_type {
                    file.push_enum_variant_tokens(
                        switch_state,
                        quote!(#variant_ident(#(#attrs)* #kind) #descriptor_tokens),
                    )?;
                } else {
                    file.push_enum_variant_tokens(
                        switch_state,
                        quote!(#variant_ident #descriptor_tokens),
                    )?;
                };
            } else {
                tracing::error!(
                    "SwitchArgs: Variant \"{variant_ident}\" not found in \"{enum_type}\"",
                );
            }
        }

        // Sort the switch enum variants by discriminant
        if let Some(item_enum) = file.get_enum_mut(&switch) {
            // Take the variants from the enum
            let mut variants: Vec<_> =
                std::mem::take(&mut item_enum.variants).into_iter().collect();

            // Sort the variants by discriminant
            variants.sort_by_key(|variant| {
                variant.discriminant.as_ref().map(|(_, expr)| match expr {
                    Expr::Lit(expr_lit) => match &expr_lit.lit {
                        Lit::Int(lit_int) => lit_int.base10_parse::<isize>().unwrap(),
                        _ => panic!("SwitchArgs: Unsupported discriminant literal"),
                    },
                    _ => panic!("SwitchArgs: Unsupported discriminant expression"),
                })
            });

            // Place the sorted variants back into the enum
            item_enum.variants = variants.into_iter().collect();
        }

        Result::kind_string(&switch)
    }

    /// Handle [`ProtocolTypeArgs::TopBitSetTerminatedArray`]
    /// [`TopBitSetTerminatedArrayArgs`]
    #[must_use]
    fn handle_bitset_array(
        state: State<'_, '_>,
        args: &TopBitSetTerminatedArrayArgs,
        file: &mut File,
    ) -> Result {
        Self::generate_type(state, &args.kind, file)
            .with_attr_tokens(quote!(#[frog(terminated)]))
            .map(|ty| format!("Vec<{ty}>"))
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

    /// Format a variant name to allow creating valid Rust [`Idents`](Ident).
    #[must_use]
    fn format_variant_name(variant_name: &str) -> String {
        variant_name.split_once(':').map_or(variant_name, |(_, split)| split).replace(['.'], "_")
    }
}
