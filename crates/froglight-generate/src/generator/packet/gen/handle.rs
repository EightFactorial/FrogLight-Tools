#[expect(unused_imports)]
use froglight_parse::file::protocol::ProtocolTypeArgs;
use froglight_parse::file::protocol::{
    ArrayArgs, ArrayWithLengthOffsetArgs, BitfieldArg, BufferArgs, ContainerArg,
    EntityMetadataArgs, MapperArgs, ProtocolType, SwitchArgs, TopBitSetTerminatedArrayArgs,
};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::LitInt;

use super::{state::Target, File, Result, State};
use crate::generator::PacketGenerator;

#[expect(unused_variables)]
impl PacketGenerator {
    /// Handle [`ProtocolTypeArgs::Array`] [`ArrayArgs`]
    #[must_use]
    pub(super) fn handle_array(state: &State<Target>, args: &ArrayArgs, file: &mut File) -> Result {
        match args {
            // Return a `Vec` of the type, and label it with the length field
            ArrayArgs::CountField { count_field, kind } => {
                let length = Self::format_field_name(count_field);
                Self::generate_type(state, kind, file)
                    .map_item(|ty| format!("Vec<{ty}>"))
                    .with_attr_tokens(quote! { #[frog(length = #length)] })
            }
            // Return a `Vec` of the type
            ArrayArgs::Count { count_type, kind } => {
                if count_type == "varint" {
                    Self::generate_type(state, kind, file).map_item(|ty| format!("Vec<{ty}>"))
                } else {
                    Result::Err(anyhow::anyhow!(
                        "ArrayArgs: Unsupported count type \"{}.{}\": \"{}\"",
                        state.item(),
                        state.target(),
                        count_type
                    ))
                }
            }
        }
    }

    /// Handle [`ProtocolTypeArgs::ArrayWithLengthOffset`]
    /// [`ArrayWithLengthOffsetArgs`]
    #[must_use]
    pub(super) fn handle_offset(
        state: &State<Target>,
        args: &ArrayWithLengthOffsetArgs,
        file: &mut File,
    ) -> Result {
        // Process the array and label it with the offset
        let offset = LitInt::new(&args.length_offset.to_string(), Span::call_site());
        Self::handle_array(state, &args.array, file)
            .with_attr_tokens(quote! { #[frog(offset = #offset)] })
    }

    /// Handle [`ProtocolTypeArgs::Bitfield`] [`BitfieldArg`]
    #[must_use]
    pub(super) fn handle_bitfield(
        state: &State<Target>,
        args: &[BitfieldArg],
        file: &mut File,
    ) -> Result {
        // Create a new struct and derive `FrogBitfield`
        let state = state.create_item();
        file.create_struct(&state);
        file.push_struct_attr_tokens(&state, quote! { #[derive(FrogBitfield)] })?;

        // Iterate over the fields
        for arg in args {
            // Create a new state for the field
            let field_state = state.with_target(Self::format_field_name(&arg.name));
            let bits = LitInt::new(&arg.size.to_string(), Span::call_site());

            // Get the field type
            let field_type = match arg.size {
                1 => "bool",
                2..=8 => "u8",
                9..=16 => "u16",
                17..=32 => "u32",
                33..=64 => "u64",
                _ => {
                    return Result::Err(anyhow::anyhow!(
                        "BitfieldArg: Unsupported size \"{}.{}\": \"{}\"",
                        field_state.item(),
                        field_state.target(),
                        arg.size
                    ))
                }
            };

            // Push the field and label it with the bit size
            file.push_struct_field_str(&field_state, field_type)?;
            file.push_struct_field_attr_tokens(&field_state, quote! { #[frog(bits = #bits)] })?;
        }

        // Return the generated struct
        Result::item_from_state(state)
    }

    /// Handle [`ProtocolTypeArgs::Buffer`] [`BufferArgs`]
    #[must_use]
    pub(super) fn handle_buffer(
        state: &State<Target>,
        args: &BufferArgs,
        file: &mut File,
    ) -> Result {
        match args {
            // Return a fixed-size array of bytes
            BufferArgs::Count(count) => Result::item_from_tokens(quote! { [u8; #count] }),
            // Return a `Vec` of bytes
            BufferArgs::CountType(count_type) => {
                if count_type == "varint" {
                    Result::item_from_tokens(quote! { Vec<u8> })
                } else {
                    Result::Err(anyhow::anyhow!(
                        "BufferArgs: Unsupported count type \"{}.{}\": \"{}\"",
                        state.item(),
                        state.target(),
                        count_type
                    ))
                }
            }
        }
    }

    /// Handle [`ProtocolTypeArgs::Container`] [`ContainerArg`]
    #[must_use]
    pub(super) fn handle_container(
        state: &State<Target>,
        args: &[ContainerArg],
        file: &mut File,
    ) -> Result {
        // Create a new struct
        let state = state.create_item();
        file.create_struct(&state);

        // Iterate over the fields
        for (index, arg) in args.iter().enumerate() {
            // Create a new state for the field
            let field_state = if let Some(name) = &arg.name {
                state.with_target(Self::format_field_name(name))
            } else {
                state.with_target(format!("field_{index}"))
            };

            // Create the field
            if let Result::Item { kind, attrs } =
                Self::generate_type(&field_state, &arg.kind, file)?
            {
                file.push_struct_field(&field_state, kind)?;
                file.push_struct_field_attrs(&field_state, attrs)?;
            }
        }

        // Return the generated struct
        Result::item_from_state(state)
    }

    /// Handle [`ProtocolTypeArgs::EntityMetadata`] [`EntityMetadataArgs`]
    #[must_use]
    pub(super) fn handle_metadata(
        state: &State<Target>,
        args: &EntityMetadataArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Mapper`] [`MapperArgs`]
    #[must_use]
    pub(super) fn handle_mapper(
        state: &State<Target>,
        args: &MapperArgs,
        file: &mut File,
    ) -> Result {
        // Create a new enum
        let state = state.create_item();
        file.create_enum(&state);

        // Collect and sort the variants
        let mut collection: Vec<_> = args.mappings.iter().collect();
        collection.sort_by_key(|(key, _)| key.parse::<isize>().unwrap());

        // Iterate over the mappings
        for (input, output) in collection {
            // Create a new state and push the variant
            let variant_state = state.with_target(Self::format_variant_name(output));
            let discriminant = LitInt::new(input, Span::call_site()).into_token_stream();
            file.push_enum_variant(&variant_state, Some(discriminant))?;
        }

        // Return the generated enum
        Result::item_from_state(state)
    }

    /// Handle [`ProtocolTypeArgs::Option`] [`ProtocolType`]
    ///
    /// Always maps to `Option<T>`
    #[must_use]
    pub(super) fn handle_option(
        state: &State<Target>,
        opt: &ProtocolType,
        file: &mut File,
    ) -> Result {
        Self::generate_type(state, opt, file).map_item(|ty| format!("Option<{ty}>"))
    }

    /// Handle [`ProtocolTypeArgs::PString`] [`BufferArgs`]
    ///
    /// Always returns [`String`]
    #[must_use]
    pub(super) fn handle_pstring(
        _state: &State<Target>,
        _args: &BufferArgs,
        _file: &mut File,
    ) -> Result {
        Result::item_from_str("string")
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    #[must_use]
    pub(super) fn handle_switch(
        state: &State<Target>,
        args: &SwitchArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }
    /// Handle [`ProtocolTypeArgs::TopBitSetTerminatedArray`]
    /// [`TopBitSetTerminatedArrayArgs`]
    #[must_use]
    pub(super) fn handle_bitset_array(
        state: &State<Target>,
        args: &TopBitSetTerminatedArrayArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }
}
