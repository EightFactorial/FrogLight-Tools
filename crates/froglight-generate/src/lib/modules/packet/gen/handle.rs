#[expect(unused_imports)]
use froglight_parse::file::protocol::ProtocolTypeArgs;
use froglight_parse::file::protocol::{
    ArrayArgs, ArrayWithLengthOffsetArgs, BitfieldArg, BitflagArgs, BufferArgs, ContainerArg,
    EntityMetadataArgs, MapperArgs, ProtocolType, RegistryEntryHolderArgs,
    RegistryEntryHolderSetArgs, SwitchArgs, TopBitSetTerminatedArrayArgs,
};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{Expr, ExprLit, Ident, Lit, LitInt};

use super::{
    state::{Item, Target},
    File, Result, State,
};
use crate::modules::PacketGenerator;

#[expect(unused_variables)]
impl PacketGenerator {
    /// Handle [`ProtocolTypeArgs::Array`] [`ArrayArgs`]
    #[must_use]
    pub(super) fn handle_array(state: &State<Target>, args: &ArrayArgs, file: &mut File) -> Result {
        match args {
            // Return a `Vec` of the type, and label it with the length field
            ArrayArgs::CountField { count_field, kind } => {
                let length = Ident::new(&Self::format_field_name(count_field), Span::call_site());
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
        // Get the count field and kind
        let (count_field, kind) = match &args.array {
            ArrayArgs::CountField { count_field, kind } => (count_field, kind),
            ArrayArgs::Count { count_type, kind } => (count_type, kind),
        };
        assert_eq!(
            count_field, "type",
            "ArrayWithLengthOffsetArgs: Invalid count field \"{count_field}\"",
        );

        // Generate the type and label it with the length offset
        let offset = LitInt::new(&args.length_offset.to_string(), Span::call_site());
        Self::generate_type(state, kind, file)?
            .with_attr_tokens(quote! { #[frog(offset = #offset)] })
            .map_item(|ty| format!("Vec<{ty}>"))
    }

    /// Handle [`ProtocolTypeArgs::Bitfield`] [`BitfieldArg`]
    #[must_use]
    pub(super) fn handle_bitfield(
        state: &State<Target>,
        args: &[BitfieldArg],
        file: &mut File,
    ) -> Result {
        // Create a new struct and add the `bitfield` attribute
        let state = state.create_item();
        file.create_struct(&state);
        file.push_struct_attr_tokens(&state, quote! { #[frog(bitfield)] })?;

        // Iterate over the fields
        for arg in args {
            // Create a new state for the field
            let field_state = state.with_target(&Self::format_field_name(&arg.name));
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

    /// Handle [`ProtocolTypeArgs::Bitflags`] [`BitflagArgs`]
    #[must_use]
    pub(super) fn handle_bitflags(
        state: &State<Target>,
        args: &BitflagArgs,
        file: &mut File,
    ) -> Result {
        // Create a new struct and add the `bitflags` attribute
        let state = state.create_item();
        file.create_struct(&state);
        file.push_struct_attr_tokens(&state, quote! { #[frog(bitflags)] })?;

        // Iterate over the fields
        for arg in &args.flags {
            // Create a new state for the field
            let field_state = state.with_target(&Self::format_field_name(arg));
            // Push the field
            file.push_struct_field_str(&field_state, "bool")?;
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
                state.with_target(&Self::format_field_name(name))
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
        for (key, value) in collection {
            // Create a new state and push the variant
            let variant_state = state.with_target(Self::format_variant_name(value));
            let discriminant = LitInt::new(key, Span::call_site()).into_token_stream();
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

    /// Handle [`ProtocolTypeArgs::RegistryEntryHolder`]
    /// [`RegistryEntryHolderArgs`]
    #[must_use]
    pub(super) fn handle_registry_entry(
        state: &State<Target>,
        entry: &RegistryEntryHolderArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::RegistryEntryHolderSet`]
    /// [`RegistryEntryHolderSetArgs`]
    #[must_use]
    pub(super) fn handle_registry_entry_set(
        state: &State<Target>,
        entry: &RegistryEntryHolderSetArgs,
        file: &mut File,
    ) -> Result {
        Result::unsupported()
    }

    /// Handle [`ProtocolTypeArgs::Switch`] [`SwitchArgs`]
    #[must_use]
    pub(super) fn handle_switch(
        state: &State<Target>,
        args: &SwitchArgs,
        file: &mut File,
    ) -> Result {
        let compared_field = Self::format_field_name(&args.compare_to);
        if compared_field.contains('/') {
            tracing::warn!(
                "SwitchArgs: \"{}.{}\" references a nested field \"{compared_field}\"",
                state.item(),
                state.target()
            );
            return Result::unsupported();
        }

        let compared_state = state.with_target(&compared_field);
        let compared_expr: Expr = syn::parse_str(&compared_field).unwrap();

        if let Some(compared_type) = file.get_struct_field_type(&compared_state) {
            match compared_type.to_string().as_str() {
                "bool" => Self::handle_switch_bool(state, args, file),
                "varint" | "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => {
                    Self::handle_switch_integer(state, args, file)
                }
                enum_type if enum_type.starts_with("__") => {
                    Self::handle_switch_enum(state, &compared_state.create_item(), args, file)
                }
                _ => Result::unsupported(),
            }
            .with_attr_tokens(quote! { #[frog(match_on = #compared_expr)] })
        } else {
            Result::Err(anyhow::anyhow!(
                "SwitchArgs: \"{}.{}\" references an unknown field \"{}.{}\"",
                state.item(),
                state.target(),
                compared_state.item(),
                compared_state.target()
            ))
        }
    }

    /// Handle [`ProtocolTypeArgs::Switch`]
    /// when the compared field is a boolean
    fn handle_switch_bool(state: &State<Target>, args: &SwitchArgs, file: &mut File) -> Result {
        // Condense this into an optional field
        if args.fields.len() == 1
            && matches!(args.fields.keys().next().unwrap().as_str(), "true" | "1" | "0x1")
            && args.default.is_none()
        {
            return Self::handle_option(state, args.fields.values().next().unwrap(), file);
        }

        // Create a new enum
        let state = state.create_item();
        file.create_enum(&state);

        // Collect and sort the variants
        let mut collection: Vec<_> = args.fields.iter().collect();
        collection.sort_by_key(|(key, _)| key.parse::<isize>().unwrap());

        // Iterate over the variants, should only be `true` and `false`
        for (key, value) in &collection {
            // Create a new state and discriminant
            let variant_state =
                state.with_target(format!("when_{}", Self::format_variant_name(key)));
            let discriminant = match key.as_str() {
                "false" | "0" | "0x0" => quote! { 0 },
                "true" | "1" | "0x1" => quote! { 1 },
                _ => unreachable!("Boolean switch with non-boolean key"),
            };

            // Push the variant, field, and attributes
            file.push_enum_variant(&variant_state, Some(discriminant))?;
            if let Result::Item { kind, attrs } = Self::generate_type(&variant_state, value, file)?
            {
                file.push_enum_variant_field_type(&variant_state, kind, attrs)?;
            }
        }

        // Push the default variant, if any
        if let Some(default) = &args.default {
            let variant_state = state.with_target("default");

            file.push_enum_variant(&variant_state, None)?;
            file.push_enum_variant_field_type(
                &variant_state,
                syn::parse_str("varint").unwrap(),
                Vec::new(),
            )?;

            if let Result::Item { kind, attrs } =
                Self::generate_type(&variant_state, default, file)?
            {
                file.push_enum_variant_field_type(&variant_state, kind, attrs)?;
            }
        }

        // Return the generated enum
        Result::item_from_state(state)
    }

    /// Handle [`ProtocolTypeArgs::Switch`]
    /// when the compared field is an integer
    fn handle_switch_integer(state: &State<Target>, args: &SwitchArgs, file: &mut File) -> Result {
        // Create a new enum
        let integer_state = state.create_item();
        file.create_enum(&integer_state);

        // Collect and sort the variants
        let mut collection: Vec<_> = args.fields.iter().collect();
        collection.sort_by_key(|(key, _)| key.parse::<isize>().unwrap());

        // Iterate over the variants
        for (key, value) in &collection {
            // Create a new state and discriminant
            let variant_state = integer_state.with_target(format!("variant_{key}"));
            let discriminant = LitInt::new(key, Span::call_site()).into_token_stream();

            // Push the variant, field, and attributes
            file.push_enum_variant(&variant_state, Some(discriminant))?;
            if let Result::Item { kind, attrs } = Self::generate_type(&variant_state, value, file)?
            {
                file.push_enum_variant_field_type(&variant_state, kind, attrs)?;
            }
        }

        // Push the default variant, if any
        if let Some(default) = &args.default {
            let variant_state = integer_state.with_target("default");

            file.push_enum_variant(&variant_state, None)?;
            file.push_enum_variant_field_type(
                &variant_state,
                syn::parse_str("varint").unwrap(),
                Vec::new(),
            )?;

            if let Result::Item { kind, attrs } =
                Self::generate_type(&variant_state, default, file)?
            {
                file.push_enum_variant_field_type(&variant_state, kind, attrs)?;
            }
        }

        // Return the generated enum
        Result::item_from_state(integer_state)
    }

    /// Handle [`ProtocolTypeArgs::Switch`]
    /// when the compared field is an enum
    fn handle_switch_enum(
        state: &State<Target>,
        referenced_state: &State<Item>,
        args: &SwitchArgs,
        file: &mut File,
    ) -> Result {
        // Create a new enum
        let enum_state = state.create_item();
        file.create_enum(&enum_state);

        // Iterate over the variants
        for (key, value) in &args.fields {
            // Create a new state and get the referenced variant
            let variant_state = enum_state.with_target(Self::format_variant_name(key));
            let Some(referenced_variant) = file
                .get_enum_variant(&referenced_state.with_target(Self::format_variant_name(key)))
            else {
                tracing::warn!(
                    "SwitchArgs: \"{}.{}\" references an unknown enum variant \"{}.{}\"",
                    state.item(),
                    state.target(),
                    variant_state.item(),
                    variant_state.target()
                );
                continue;
            };

            // Push the enum variant
            let discriminant = referenced_variant.discriminant.as_ref().map(|(_, d)| quote!(#d));
            file.push_enum_variant(&variant_state, discriminant)?;

            // Push the field and attributes, if any
            if let Result::Item { kind, attrs } = Self::generate_type(&variant_state, value, file)?
            {
                file.push_enum_variant_field_type(&variant_state, kind, attrs)?;
            }
        }

        // Sort the generated variants
        if let Some(item_enum) = file.get_enum_mut(&enum_state) {
            let mut variants: Vec<_> =
                std::mem::take(&mut item_enum.variants).into_iter().collect();

            variants.sort_by_key(|variant| {
                variant.discriminant.as_ref().and_then(|(_, d)| {
                    if let Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) = d {
                        lit.base10_parse::<isize>().ok()
                    } else {
                        None
                    }
                })
            });

            item_enum.variants.extend(variants);
        }

        // Push the default variant, if any
        if let Some(default) = &args.default {
            let variant_state = enum_state.with_target("default");

            file.push_enum_variant(&variant_state, None)?;
            file.push_enum_variant_field_type(
                &variant_state,
                syn::parse_str("varint").unwrap(),
                Vec::new(),
            )?;

            if let Result::Item { kind, attrs } =
                Self::generate_type(&variant_state, default, file)?
            {
                file.push_enum_variant_field_type(&variant_state, kind, attrs)?;
            }
        }

        // Return the generated enum
        Result::item_from_state(enum_state)
    }

    /// Handle [`ProtocolTypeArgs::TopBitSetTerminatedArray`]
    /// [`TopBitSetTerminatedArrayArgs`]
    #[must_use]
    pub(super) fn handle_bitset_array(
        state: &State<Target>,
        args: &TopBitSetTerminatedArrayArgs,
        file: &mut File,
    ) -> Result {
        Self::generate_type(state, &args.kind, file).map_item(|ty| format!("BitVec<{ty}>"))
    }
}
