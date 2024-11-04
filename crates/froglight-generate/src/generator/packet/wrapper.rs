use derive_more::derive::{From, Into};
use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token, Field, FieldMutability, Fields, FieldsNamed, File, Generics,
    Ident, Item, ItemEnum, ItemStruct, Token, Type, Variant, Visibility,
};

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

    /// Get the last [`Item`] in the [`FileWrapper`]
    ///
    /// # Panics
    /// This method will panic if the [`FileWrapper`] is empty.
    #[must_use]
    pub fn last(&self) -> &Item { self.0.items.last().expect("FileWrapper is empty!") }
    /// Get the last [`Item`] in the [`FileWrapper`] mutably
    ///
    /// # Panics
    /// This method will panic if the [`FileWrapper`] is empty.
    #[must_use]
    pub fn last_mut(&mut self) -> &mut Item {
        self.0.items.last_mut().expect("FileWrapper is empty!")
    }

    /// Get the last [`Ident`] in the [`FileWrapper`]
    ///
    /// # Panics
    /// This method will panic if the [`FileWrapper`] is empty,
    /// or if the last [`Item`] is not an [`ItemEnum`] or [`ItemStruct`].
    pub fn last_ident(&self) -> &Ident {
        match self.last() {
            Item::Enum(item_enum) => &item_enum.ident,
            Item::Struct(item_struct) => &item_struct.ident,
            _ => panic!("Last item is not an ItemEnum or ItemStruct"),
        }
    }
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
}

#[expect(clippy::unused_self)]
impl FileWrapper {
    /// Resolve a field reference
    pub fn resolve_field_ref(&self, _field: &str) -> Option<&Field> { None }

    /// Resolve a mutable field reference
    pub fn resolve_field_mut(&mut self, _field: &str) -> Option<&mut Field> { None }

    /// Resolve a field type
    pub fn resolve_field_type(&self, field: &str) -> Option<&Ident> {
        self.resolve_field_ref(field).map(|field_ref| match &field_ref.ty {
            Type::Path(type_path) => {
                &type_path.path.segments.last().expect("Type::Path has no segments").ident
            }
            _ => panic!("Field type is not a Type::Path: {:?} -> {field}", field_ref.ident),
        })
    }
}
