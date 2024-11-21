use derive_more::derive::{From, Into};
use proc_macro2::{Span, TokenStream};
use syn::{
    punctuated::Punctuated, token, Attribute, Field, Fields, Generics, Ident, Item, ItemEnum,
    ItemStruct, Token, Type, Variant, Visibility,
};

use super::{
    state::{Item as StateItem, Target as StateTarget},
    State,
};

#[derive(From, Into)]
pub struct File(syn::File);

impl Default for File {
    fn default() -> Self { Self(syn::File { shebang: None, attrs: Vec::new(), items: Vec::new() }) }
}

impl File {
    /// Create a new, empty [`File`].
    #[must_use]
    pub fn new() -> Self { Self::default() }

    /// Get the inner [`syn::File`].
    #[must_use]
    pub fn into_inner(self) -> syn::File { self.0 }
}

#[expect(dead_code)]
impl File {
    /// Get an [`Item`] from the [`File`].
    pub(crate) fn get_item(&self, state: &State<StateItem>) -> Option<&Item> {
        self.get_item_internal(state.item())
    }

    fn get_item_internal(&self, ident: &Ident) -> Option<&Item> {
        self.0.items.iter().find(|item| match item {
            Item::Struct(item) => &item.ident == ident,
            Item::Enum(item) => &item.ident == ident,
            _ => false,
        })
    }

    /// Get a mutable [`Item`] from the [`File`].
    pub(crate) fn get_item_mut(&mut self, state: &State<StateItem>) -> Option<&mut Item> {
        self.get_item_mut_internal(state.item())
    }

    fn get_item_mut_internal(&mut self, ident: &Ident) -> Option<&mut Item> {
        self.0.items.iter_mut().find(|item| match item {
            Item::Struct(item) => &item.ident == ident,
            Item::Enum(item) => &item.ident == ident,
            _ => false,
        })
    }

    /// Get a [`ItemStruct`] from the [`File`].
    pub(crate) fn get_struct(&self, state: &State<StateItem>) -> Option<&ItemStruct> {
        self.get_item(state).and_then(|item| match item {
            Item::Struct(item) => Some(item),
            _ => None,
        })
    }

    /// Get a mutable [`ItemStruct`] from the [`File`].
    pub(crate) fn get_struct_mut(&mut self, state: &State<StateItem>) -> Option<&mut ItemStruct> {
        self.get_item_mut(state).and_then(|item| match item {
            Item::Struct(item) => Some(item),
            _ => None,
        })
    }

    /// Get a [`ItemEnum`] from the [`File`].
    pub(crate) fn get_enum(&self, state: &State<StateItem>) -> Option<&ItemEnum> {
        self.get_item(state).and_then(|item| match item {
            Item::Enum(item) => Some(item),
            _ => None,
        })
    }

    /// Get a mutable [`ItemEnum`] from the [`File`].
    pub(crate) fn get_enum_mut(&mut self, state: &State<StateItem>) -> Option<&mut ItemEnum> {
        self.get_item_mut(state).and_then(|item| match item {
            Item::Enum(item) => Some(item),
            _ => None,
        })
    }
}

impl File {
    /// Create a new [`ItemStruct`] in the [`File`].
    pub fn create_struct(&mut self, state: &State<StateItem>) {
        self.create_struct_internal(state.item());
    }

    fn create_struct_internal(&mut self, ident: &Ident) {
        self.0.items.push(Item::Struct(ItemStruct {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(Token![pub](Span::call_site())),
            struct_token: Token![struct](Span::call_site()),
            ident: ident.clone(),
            generics: Generics::default(),
            fields: Fields::Unit,
            semi_token: Some(Token![;](Span::call_site())),
        }));
    }

    /// Push an [`Attribute`] into an [`ItemStruct`].
    pub(crate) fn push_struct_attr(
        &mut self,
        state: &State<StateItem>,
        attr: Attribute,
    ) -> anyhow::Result<()> {
        if let Some(Item::Struct(item)) = self.get_item_mut_internal(state.item()) {
            item.attrs.push(attr);
            Ok(())
        } else {
            anyhow::bail!(
                "File: Tried to push an attribute to a non-struct item, \"{}\"",
                state.item()
            );
        }
    }

    /// Push an [`Attribute`] into an [`ItemStruct`].
    pub(crate) fn push_struct_attr_tokens(
        &mut self,
        state: &State<StateItem>,
        tokens: TokenStream,
    ) -> anyhow::Result<()> {
        self.push_struct_attr(state, syn::parse_quote!(#tokens))
    }
}

impl File {
    /// Get a [`Field`] from an [`ItemStruct`].
    pub(crate) fn get_struct_field(&self, state: &State<StateTarget>) -> Option<&Field> {
        self.get_item_internal(state.item()).and_then(|item| match item {
            Item::Struct(item) => item
                .fields
                .iter()
                .find(|field| field.ident.as_ref().is_some_and(|field| field == state.target())),
            _ => None,
        })
    }

    /// Get the [`Type`]'s [`Ident`] from a [`Field`] in an [`ItemStruct`].
    pub(crate) fn get_struct_field_type(&self, state: &State<StateTarget>) -> Option<&Ident> {
        self.get_struct_field(state).and_then(|field| match &field.ty {
            Type::Path(path) => path.path.segments.last().map(|segment| &segment.ident),
            _ => None,
        })
    }

    /// Get a mutable [`Field`] from an [`ItemStruct`].
    pub(crate) fn get_struct_field_mut(
        &mut self,
        state: &State<StateTarget>,
    ) -> Option<&mut Field> {
        self.get_item_mut_internal(state.item()).and_then(|item| match item {
            Item::Struct(item) => item
                .fields
                .iter_mut()
                .find(|field| field.ident.as_ref().is_some_and(|field| field == state.target())),
            _ => None,
        })
    }

    /// Push a [`Field`] into an [`ItemStruct`].
    ///
    /// Will convert the [`ItemStruct`] `fields`
    /// into a [`Fields::Named`] if it is a [`Fields::Unit`].
    pub(crate) fn push_struct_field(
        &mut self,
        state: &State<StateTarget>,
        kind: Type,
    ) -> anyhow::Result<()> {
        if let Some(Item::Struct(ItemStruct { fields, .. })) =
            self.get_item_mut_internal(state.item())
        {
            match fields {
                Fields::Named(fields) => {
                    let field = state.target();
                    fields.named.push(syn::parse_quote!(pub #field: #kind));
                }
                Fields::Unnamed(fields) => {
                    fields.unnamed.push(syn::parse_quote!(pub #kind));
                }
                Fields::Unit => {
                    let field = state.target();
                    *fields = Fields::Named(syn::parse_quote!({ pub #field: #kind }));
                }
            }
            Ok(())
        } else {
            anyhow::bail!(
                "File: Tried to push a field to a non-struct item, \"{}.{}\"",
                state.item(),
                state.target()
            );
        }
    }

    /// Push a [`Field`] into an [`ItemStruct`].
    pub(crate) fn push_struct_field_str(
        &mut self,
        state: &State<StateTarget>,
        kind: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        match syn::parse_str(kind.as_ref()) {
            Ok(kind) => self.push_struct_field(state, kind),
            Err(err) => anyhow::bail!(
                "File: Failed to parse type \"{}\" for field \"{}.{}\": {}",
                kind.as_ref(),
                state.item(),
                state.target(),
                err
            ),
        }
    }

    /// Push an [`Attribute`] into a [`Field`] in an [`ItemStruct`].
    pub(crate) fn push_struct_field_attr(
        &mut self,
        state: &State<StateTarget>,
        attr: Attribute,
    ) -> anyhow::Result<()> {
        if let Some(field) = self.get_struct_field_mut(state) {
            field.attrs.push(attr);
            Ok(())
        } else {
            anyhow::bail!(
                "File: Tried to push an attribute to a non-existent field, \"{}.{}\"",
                state.item(),
                state.target()
            );
        }
    }

    /// Push an [`Attribute`] into a [`Field`] in an [`ItemStruct`].
    pub(crate) fn push_struct_field_attr_tokens(
        &mut self,
        state: &State<StateTarget>,
        tokens: TokenStream,
    ) -> anyhow::Result<()> {
        self.push_struct_field_attr(state, syn::parse_quote!(#tokens))
    }

    /// Push multiple [`Attribute`]s into a [`Field`] in an [`ItemStruct`].
    pub(crate) fn push_struct_field_attrs(
        &mut self,
        state: &State<StateTarget>,
        attrs: impl IntoIterator<Item = Attribute>,
    ) -> anyhow::Result<()> {
        if let Some(field) = self.get_struct_field_mut(state) {
            field.attrs.extend(attrs);
            Ok(())
        } else {
            anyhow::bail!(
                "File: Tried to push attributes to a non-existent field, \"{}.{}\"",
                state.item(),
                state.target()
            );
        }
    }
}

impl File {
    /// Create a new [`ItemEnum`] in the [`File`].
    pub fn create_enum(&mut self, state: &State<StateItem>) {
        self.create_enum_internal(state.item());
    }

    fn create_enum_internal(&mut self, ident: &Ident) {
        self.0.items.push(Item::Enum(ItemEnum {
            attrs: Vec::new(),
            vis: Visibility::Public(Token![pub](Span::call_site())),
            enum_token: Token![enum](Span::call_site()),
            ident: ident.clone(),
            generics: Generics::default(),
            brace_token: token::Brace::default(),
            variants: Punctuated::new(),
        }));
    }
}

#[expect(dead_code)]
impl File {
    /// Get a [`Variant`] from an [`ItemEnum`].
    pub(crate) fn get_enum_variant(&self, state: &State<StateTarget>) -> Option<&Variant> {
        self.get_item_internal(state.item()).and_then(|item| match item {
            Item::Enum(item) => {
                item.variants.iter().find(|variant| &variant.ident == state.target())
            }
            _ => None,
        })
    }

    /// Get a mutable [`Variant`] from an [`ItemEnum`].
    pub(crate) fn get_enum_variant_mut(
        &mut self,
        state: &State<StateTarget>,
    ) -> Option<&mut Variant> {
        self.get_item_mut_internal(state.item()).and_then(|item| match item {
            Item::Enum(item) => {
                item.variants.iter_mut().find(|variant| &variant.ident == state.target())
            }
            _ => None,
        })
    }

    /// Push a [`Variant`] into an [`ItemEnum`].
    pub(crate) fn push_enum_variant(
        &mut self,
        state: &State<StateTarget>,
        discriminant: Option<TokenStream>,
    ) -> anyhow::Result<()> {
        if let Some(Item::Enum(ItemEnum { variants, .. })) =
            self.get_item_mut_internal(state.item())
        {
            let target = state.target();
            if let Some(des) = discriminant {
                variants.push(syn::parse_quote!(#target = #des));
            } else {
                variants.push(syn::parse_quote!(#target));
            }
            Ok(())
        } else {
            anyhow::bail!(
                "File: Tried to push a variant to a non-enum item, \"{}.{}\"",
                state.item(),
                state.target()
            );
        }
    }

    /// Get a [`Field`] from a [`Variant`] in an [`ItemEnum`].
    pub(crate) fn get_enum_variant_field(
        &self,
        state: &State<StateTarget>,
        ident: &Ident,
    ) -> Option<&Field> {
        self.get_enum_variant(state).and_then(|variant| {
            variant.fields.iter().find(|field| {
                if let Some(field_ident) = field.ident.as_ref() {
                    // First, check if `ident` matches a field name
                    field_ident == ident
                } else if let Type::Path(field_type_path) = &field.ty {
                    // If not, check if `ident` matches a field type
                    if let Some(last) = field_type_path.path.segments.last() {
                        last.ident == *ident
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        })
    }

    /// Get a mutable [`Field`] from a [`Variant`] in an [`ItemEnum`].
    pub(crate) fn get_enum_variant_field_mut(
        &mut self,
        state: &State<StateTarget>,
        ident: &Ident,
    ) -> Option<&mut Field> {
        self.get_enum_variant_mut(state).and_then(|variant| {
            variant.fields.iter_mut().find(|field| {
                if let Some(field_ident) = field.ident.as_ref() {
                    // First, check if `ident` matches a field name
                    field_ident == ident
                } else if let Type::Path(field_type_path) = &field.ty {
                    // If not, check if `ident` matches a field type
                    if let Some(last) = field_type_path.path.segments.last() {
                        last.ident == *ident
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
        })
    }

    pub(crate) fn push_enum_variant_field(
        &mut self,
        state: &State<StateTarget>,
        field: Field,
    ) -> anyhow::Result<()> {
        if let Some(variant) = self.get_enum_variant_mut(state) {
            match &mut variant.fields {
                Fields::Named(fields) => {
                    fields.named.push(field);
                }
                Fields::Unnamed(fields) => {
                    fields.unnamed.push(field);
                }
                Fields::Unit => {
                    variant.fields = if field.ident.is_some() {
                        Fields::Named(syn::parse_quote!({#field}))
                    } else {
                        Fields::Unnamed(syn::parse_quote!((#field)))
                    }
                }
            }
            Ok(())
        } else {
            anyhow::bail!(
                "File: Tried to push a field to a non-existent variant, \"{}.{}\"",
                state.item(),
                state.target()
            );
        }
    }

    /// Push a [`Field`] into a [`Variant`] in an [`ItemEnum`].
    pub(crate) fn push_enum_variant_field_type(
        &mut self,
        state: &State<StateTarget>,
        kind: Type,
        attrs: impl IntoIterator<Item = Attribute>,
    ) -> anyhow::Result<()> {
        let attrs = attrs.into_iter();
        self.push_enum_variant_field(state, syn::parse_quote!(#(#attrs)* #kind))
    }
}
