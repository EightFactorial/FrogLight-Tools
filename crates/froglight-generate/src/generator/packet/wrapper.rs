use derive_more::derive::{From, Into};
use froglight_parse::file::protocol::SwitchArgs;
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token, Field, FieldMutability, Fields, FieldsNamed, FieldsUnnamed,
    File, Generics, Ident, Item, ItemEnum, ItemStruct, Token, Type, Variant, Visibility,
};

use super::PacketGenerator;

#[derive(From, Into)]
pub struct FileWrapper(File);

impl FileWrapper {
    #[must_use]
    pub fn new() -> Self { Self::default() }
    #[must_use]
    pub fn into_inner(self) -> File { self.0 }
}

impl Default for FileWrapper {
    fn default() -> Self { Self(File { shebang: None, attrs: Vec::new(), items: Vec::new() }) }
}

impl FileWrapper {
    /// Push an [`Item`] into the [`FileWrapper`]
    pub fn push(&mut self, item: impl Into<Item>) { self.0.items.push(item.into()); }
}

impl FileWrapper {
    /// Get an [`Item`] by its identifier
    pub fn get_item(&self, ident: &str) -> Option<&Item> {
        self.0.items.iter().find(|item| match item {
            Item::Enum(item_enum) => item_enum.ident == ident,
            Item::Mod(item_mod) => item_mod.ident == ident,
            _ => false,
        })
    }

    /// Get a mutable [`Item`] by its identifier
    pub fn get_item_mut(&mut self, ident: &str) -> Option<&mut Item> {
        self.0.items.iter_mut().find(|item| match item {
            Item::Enum(item_enum) => item_enum.ident == ident,
            Item::Mod(item_mod) => item_mod.ident == ident,
            _ => false,
        })
    }

    /// Get an [`ItemStruct`] by its identifier
    pub fn get_struct(&self, ident: &str) -> Option<&ItemStruct> {
        self.0.items.iter().find_map(|item| match item {
            Item::Struct(item_struct) if item_struct.ident == ident => Some(item_struct),
            _ => None,
        })
    }

    /// Get a [`Field`] from a [`ItemStruct`] by its identifier
    pub fn get_struct_field(&self, struct_ident: &str, field_ident: &str) -> Option<&Field> {
        self.get_struct(struct_ident).and_then(|item_struct| {
            item_struct
                .fields
                .iter()
                .find(|&field| field.ident.as_ref().is_some_and(|ident| ident == field_ident))
        })
    }

    /// Get a [`ItemStruct`]'s [`FieldsNamed`] by its identifier
    pub fn get_struct_fields(&self, struct_ident: &str) -> Option<&FieldsNamed> {
        self.get_struct(struct_ident).and_then(|item_struct| {
            if let Fields::Named(fields) = &item_struct.fields {
                Some(fields)
            } else {
                None
            }
        })
    }

    /// Get a mutable [`ItemStruct`] by its identifier
    pub fn get_struct_mut(&mut self, ident: &str) -> Option<&mut ItemStruct> {
        self.0.items.iter_mut().find_map(|item| match item {
            Item::Struct(item_struct) if item_struct.ident == ident => Some(item_struct),
            _ => None,
        })
    }

    /// Get a mutable [`ItemStruct`]'s [`FieldsNamed`] by its identifier
    pub fn get_struct_fields_mut(&mut self, struct_ident: &str) -> Option<&mut FieldsNamed> {
        self.get_struct_mut(struct_ident).and_then(|item_struct| {
            if let Fields::Named(fields) = &mut item_struct.fields {
                Some(fields)
            } else {
                None
            }
        })
    }

    /// Get a [`Field`] from a mutable [`ItemStruct`] by its identifier
    pub fn get_struct_field_mut(
        &mut self,
        struct_ident: &str,
        field_ident: &str,
    ) -> Option<&mut Field> {
        self.get_struct_mut(struct_ident).and_then(|item_struct| {
            item_struct
                .fields
                .iter_mut()
                .find(|field| field.ident.as_ref().is_some_and(|ident| ident == field_ident))
        })
    }

    /// Get an [`ItemEnum`] by its identifier
    pub fn get_enum(&self, ident: &str) -> Option<&ItemEnum> {
        self.0.items.iter().find_map(|item| match item {
            Item::Enum(item_enum) if item_enum.ident == ident => Some(item_enum),
            _ => None,
        })
    }

    /// Get a [`Variant`] from an [`ItemEnum`] by its identifier
    pub fn get_enum_variant(&self, enum_ident: &str, variant_ident: &str) -> Option<&Variant> {
        self.get_enum(enum_ident).and_then(|item_enum| {
            item_enum.variants.iter().find(|&variant| variant.ident == variant_ident)
        })
    }

    /// Get a mutable [`ItemEnum`] by its identifier
    pub fn get_enum_mut(&mut self, ident: &str) -> Option<&mut ItemEnum> {
        self.0.items.iter_mut().find_map(|item| match item {
            Item::Enum(item_enum) if item_enum.ident == ident => Some(item_enum),
            _ => None,
        })
    }

    /// Get a [`Variant`] from a mutable [`ItemEnum`] by its identifier
    pub fn get_enum_variant_mut(
        &mut self,
        enum_ident: &str,
        variant_ident: &str,
    ) -> Option<&mut Variant> {
        self.get_enum_mut(enum_ident).and_then(|item_enum| {
            item_enum.variants.iter_mut().find(|variant| variant.ident == variant_ident)
        })
    }
}

impl FileWrapper {
    /// Create a new [`ItemStruct`] with the given identifier.
    pub fn push_struct(&mut self, ident: &str) {
        self.push(Item::Struct(ItemStruct {
            attrs: Vec::new(),
            vis: Visibility::Public(<Token![pub]>::default()),
            struct_token: <Token![struct]>::default(),
            ident: Ident::new(ident, Span::call_site()),
            generics: Generics::default(),
            fields: Fields::Named(FieldsNamed {
                brace_token: token::Brace::default(),
                named: Punctuated::new(),
            }),
            semi_token: None,
        }));
    }

    /// Push a new [`Field`] into a [`ItemStruct`] with the given identifier.
    pub fn push_field(&mut self, struct_ident: &str, field_ident: &str, field_type: &str) {
        if let Some(fields) = self.get_struct_fields_mut(struct_ident) {
            fields.named.push(Field {
                attrs: Vec::new(),
                vis: Visibility::Public(<Token![pub]>::default()),
                mutability: FieldMutability::None,
                ident: Some(Ident::new(field_ident, Span::call_site())),
                colon_token: None,
                ty: syn::parse_str(field_type).expect("Failed to parse type"),
            });
        }
    }

    /// Create a new [`ItemEnum`] with the given identifier.
    pub fn push_enum(&mut self, ident: &str) {
        self.push(Item::Enum(ItemEnum {
            attrs: Vec::new(),
            vis: Visibility::Public(<Token![pub]>::default()),
            enum_token: <Token![enum]>::default(),
            ident: Ident::new(ident, Span::call_site()),
            generics: Generics::default(),
            brace_token: token::Brace::default(),
            variants: Punctuated::new(),
        }));
    }

    /// Push a new [`Variant`] into an [`ItemEnum`] with the given identifier.
    pub fn push_variant(
        &mut self,
        enum_ident: &str,
        variant_ident: &str,
        variant_desc: Option<&str>,
    ) {
        if let Some(item_enum) = self.get_enum_mut(enum_ident) {
            item_enum.variants.push(Variant {
                attrs: Vec::new(),
                ident: Ident::new(variant_ident, Span::call_site()),
                fields: Fields::Unit,
                discriminant: variant_desc.map(|desc| {
                    (
                        <Token![=]>::default(),
                        syn::parse_str(desc).expect("Failed to parse discriminant"),
                    )
                }),
            });
        }
    }

    pub fn push_variant_field(
        &mut self,
        enum_ident: &str,
        variant_ident: &str,
        field_type: &str,
        variant_desc: Option<&str>,
    ) {
        // If the variant does not exist, create it.
        if self.get_enum_variant(enum_ident, variant_ident).is_none() {
            self.push_variant(enum_ident, variant_ident, variant_desc);
        }

        // Push the field into the variant.
        if let Some(variant) = self.get_enum_variant_mut(enum_ident, variant_ident) {
            match &mut variant.fields {
                // Push a new named field into the variant.
                Fields::Named(fields_named) => {
                    fields_named.named.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Public(<Token![pub]>::default()),
                        mutability: FieldMutability::None,
                        ident: Some(Ident::new(
                            &PacketGenerator::format_item_name(field_type),
                            Span::call_site(),
                        )),
                        colon_token: Some(<Token![:]>::default()),
                        ty: syn::parse_str(field_type).expect("Failed to parse type"),
                    });
                }
                // Push a new unnamed field into the variant.
                Fields::Unnamed(fields_unnamed) => {
                    fields_unnamed.unnamed.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Public(<Token![pub]>::default()),
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: syn::parse_str(field_type).expect("Failed to parse type"),
                    });
                }
                // Convert the unit variant into an unnamed field variant,
                // then push the new field into the variant.
                Fields::Unit => {
                    let mut unnamed = Punctuated::new();
                    unnamed.push(Field {
                        attrs: Vec::new(),
                        vis: Visibility::Public(<Token![pub]>::default()),
                        mutability: FieldMutability::None,
                        ident: None,
                        colon_token: None,
                        ty: syn::parse_str(field_type).expect("Failed to parse type"),
                    });

                    variant.fields = Fields::Unnamed(FieldsUnnamed {
                        paren_token: token::Paren::default(),
                        unnamed,
                    });
                }
            }
        }
    }
}

impl FileWrapper {
    /// Resolve a field reference
    pub fn resolve_field_ref(
        &self,
        struct_ident: &str,
        field_ident: &str,
    ) -> anyhow::Result<&Field> {
        let resolved = self.resolve_relative_field(struct_ident, field_ident)?;
        let last = PacketGenerator::format_field_name(field_ident.split('/').last().unwrap());
        self.get_struct_field(&resolved, &last).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to resolve field reference: {struct_ident} -> {resolved}.{last}"
            )
        })
    }

    /// Resolve a field type
    pub fn resolve_field_type(
        &self,
        struct_ident: &str,
        field_ident: &str,
    ) -> anyhow::Result<&Ident> {
        let field_ref = self.resolve_field_ref(struct_ident, field_ident)?;
        match &field_ref.ty {
            Type::Path(type_path) => {
                Ok(&type_path.path.segments.last().expect("Type::Path has no segments").ident)
            }
            _ => anyhow::bail!("Field type is not a Type::Path: {struct_ident} -> {field_ident}"),
        }
    }

    /// Resolve a mutable field reference
    pub fn resolve_field_mut(
        &mut self,
        struct_ident: &str,
        field_ident: &str,
    ) -> anyhow::Result<&mut Field> {
        let resolved = self.resolve_relative_field(struct_ident, field_ident)?;
        let last = PacketGenerator::format_field_name(field_ident.split('/').last().unwrap());
        self.get_struct_field_mut(&resolved, &last).ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to resolve field reference: {struct_ident} -> {resolved}.{last}"
            )
        })
    }

    fn resolve_relative_field(
        &self,
        struct_ident: &str,
        mut field_ident: &str,
    ) -> anyhow::Result<String> {
        // Track the current struct by name
        let mut struct_ident = struct_ident.to_string();

        // If the struct contains the field, return the struct
        if self.get_struct_field(&struct_ident, field_ident).is_some() {
            return Ok(struct_ident);
        }

        // Strip any "../" prefixes from the field identifier
        while let Some(stripped) = field_ident.strip_prefix("../") {
            // Find the last struct that references the current struct and get the field
            if let Some(struct_field) = self.0.items.iter().rev().find_map(|item| match item {
                Item::Struct(item_struct) => item_struct.fields.iter().find(|field| {
                    field
                        .ident
                        .as_ref()
                        .is_some_and(|ident| ident == &PacketGenerator::format_field_name(stripped))
                }),
                _ => None,
            }) {
                // Get the Ident of the field's type
                if let Type::Path(field_type) = &struct_field.ty {
                    let last = field_type.path.segments.last().expect("Type::Path has no segments");
                    // Update the current struct and field
                    struct_ident = last.ident.to_string();
                    field_ident = stripped;
                } else {
                    anyhow::bail!(
                        "Failed to resolve field reference upwards: {struct_ident} -> {field_ident}"
                    );
                }
            } else {
                anyhow::bail!(
                    "Failed to resolve field reference upwards: {struct_ident} -> {field_ident}"
                );
            }
        }

        // Follow the field's path
        let split: Vec<&str> = field_ident.split('/').collect();
        for field_name in split.iter().take(split.len().saturating_sub(1)) {
            let field_name = PacketGenerator::format_field_name(field_name);

            if let Some(struct_field) = self.get_struct_field(&struct_ident, &field_name) {
                // Get the Ident of the field's type
                if let Type::Path(field_type) = &struct_field.ty {
                    let last = field_type.path.segments.last().expect("Type::Path has no segments");
                    // Update the current struct and field
                    struct_ident = last.ident.to_string();
                }
            } else {
                anyhow::bail!(
                    "Failed to resolve field reference downwards: {struct_ident} -> {field_ident}"
                );
            }
        }

        Ok(struct_ident)
    }
}

impl FileWrapper {
    pub fn convert_or_modify_enum(
        &mut self,
        struct_ident: &str,
        switch_args: &SwitchArgs,
    ) -> anyhow::Result<()> {
        let field_type = self.resolve_field_type(struct_ident, &switch_args.compare_to)?;
        match field_type.to_string().as_str() {
            "bool" => self.create_bool_enum(struct_ident, switch_args),
            "VarInt" | "u8" | "u16" | "u32" | "i8" | "i16" | "i32" => {
                self.create_int_enum(struct_ident, switch_args)
            }
            existing => self.extend_existing_enum(existing, switch_args),
        }
    }

    fn create_bool_enum(
        &mut self,
        struct_ident: &str,
        switch_args: &SwitchArgs,
    ) -> anyhow::Result<()> {
        // Create an Enum
        let enum_ident =
            struct_ident.to_string() + &PacketGenerator::format_item_name(&switch_args.compare_to);
        self.push_enum(&enum_ident);

        // Sort the variants
        let mut collection: Vec<_> = switch_args.fields.iter().collect();
        collection
            .sort_by_key(|(key, _)| matches!(key.to_ascii_lowercase().as_str(), "true" | "0"));

        // Push the variants into the Enum
        for (index, (switch_case, switch_type)) in collection.into_iter().enumerate() {
            let mut case_ident = PacketGenerator::format_item_name(switch_case);
            if case_ident.chars().next().unwrap_or_default().is_numeric() {
                case_ident = enum_ident.clone() + &case_ident;
            }

            if let Some(case_type) =
                PacketGenerator::generate_type(&enum_ident, &case_ident, switch_type, self)?
            {
                self.push_variant_field(
                    &enum_ident,
                    &case_ident,
                    &case_type,
                    Some(&index.to_string()),
                );
            } else {
                self.push_variant(&enum_ident, &case_ident, Some(&index.to_string()));
            }
        }

        // Modify the field to use the Enum
        let compared_field = self.resolve_field_mut(struct_ident, &switch_args.compare_to)?;
        compared_field.ty = syn::parse_str(&enum_ident).expect("Invalid Enum: \"{enum_ident}\"");

        Ok(())
    }

    fn create_int_enum(
        &mut self,
        struct_ident: &str,
        switch_args: &SwitchArgs,
    ) -> anyhow::Result<()> {
        // Create an Enum
        let enum_ident =
            struct_ident.to_string() + &PacketGenerator::format_item_name(&switch_args.compare_to);
        self.push_enum(&enum_ident);

        // Sort the variants
        let mut collection: Vec<_> = switch_args.fields.iter().collect();
        collection
            .sort_by_key(|(key, _)| key.parse::<i32>().expect("Invalid Integer Enum Discriminant"));

        // Push the variants into the Enum
        for (index, (switch_case, switch_type)) in collection.into_iter().enumerate() {
            let mut case_ident = PacketGenerator::format_item_name(switch_case);
            if case_ident.chars().next().unwrap_or_default().is_numeric() {
                case_ident = enum_ident.clone() + &case_ident;
            }

            if let Some(case_type) =
                PacketGenerator::generate_type(&enum_ident, &case_ident, switch_type, self)?
            {
                self.push_variant_field(
                    &enum_ident,
                    &case_ident,
                    &case_type,
                    Some(&index.to_string()),
                );
            } else {
                self.push_variant(&enum_ident, &case_ident, Some(&index.to_string()));
            }
        }

        // Modify the field to use the Enum
        let compared_field = self.resolve_field_mut(struct_ident, &switch_args.compare_to)?;
        compared_field.ty = syn::parse_str(&enum_ident).expect("Invalid Enum: \"{enum_ident}\"");

        Ok(())
    }

    fn extend_existing_enum(
        &mut self,
        existing: &str,
        switch_args: &SwitchArgs,
    ) -> anyhow::Result<()> {
        for (index, (switch_case, switch_type)) in switch_args.fields.iter().enumerate() {
            let mut case_ident = PacketGenerator::format_item_name(switch_case);
            if case_ident.chars().next().unwrap_or_default().is_numeric() {
                case_ident = existing.to_string() + &case_ident;
            }

            if let Some(case_type) =
                PacketGenerator::generate_type(existing, &case_ident, switch_type, self)?
            {
                self.push_variant_field(
                    existing,
                    &case_ident,
                    &case_type,
                    Some(&index.to_string()),
                );
            } else {
                self.push_variant(existing, &case_ident, Some(&index.to_string()));
            }
        }
        Ok(())
    }
}
