use derive_more::derive::{From, Into};
use proc_macro2::{Span, TokenStream};
use syn::{
    punctuated::Punctuated, token::Brace, Attribute, Field, Fields, Generics, Ident, Item,
    ItemEnum, ItemStruct, Token, Type, Variant, Visibility,
};

use super::state::State;

#[derive(From, Into)]
pub struct File(syn::File);

impl File {
    #[must_use]
    pub fn new() -> Self { Self::default() }
    #[must_use]
    pub fn into_inner(self) -> syn::File { self.0 }
}

impl Default for File {
    fn default() -> Self { Self(syn::File { shebang: None, attrs: Vec::new(), items: Vec::new() }) }
}

impl File {
    /// Push an [`Item`] into the [`File`].
    pub fn push_item(&mut self, item: Item) { self.0.items.push(item); }

    /// Push an [`ItemStruct`] into the [`File`].
    pub fn push_struct(&mut self, item: ItemStruct) { self.push_item(Item::Struct(item)); }

    /// Push an [`ItemEnum`] into the [`File`].
    pub fn push_enum(&mut self, item: ItemEnum) { self.push_item(Item::Enum(item)); }

    /// Get an [`Item`] from the [`File`] by [`Ident`].
    pub fn get_item(&self, ident: &Ident) -> Option<&Item> {
        self.0.items.iter().find(|item| match item {
            Item::Enum(item_enum) => item_enum.ident == *ident,
            Item::Struct(item_struct) => item_struct.ident == *ident,
            _ => false,
        })
    }

    /// Get a mutable [`Item`] from the [`File`] by [`Ident`].
    pub fn get_item_mut(&mut self, ident: &Ident) -> Option<&mut Item> {
        self.0.items.iter_mut().find(|item| match item {
            Item::Enum(item_enum) => item_enum.ident == *ident,
            Item::Struct(item_struct) => item_struct.ident == *ident,
            _ => false,
        })
    }

    /// Get an [`ItemStruct`] from the [`File`] by [`Ident`].
    pub fn get_struct(&self, ident: &Ident) -> Option<&ItemStruct> {
        self.get_item(ident).and_then(|item| match item {
            Item::Struct(item_struct) => Some(item_struct),
            _ => None,
        })
    }

    /// Get a mutable [`ItemStruct`] from the [`File`] by [`Ident`].
    pub fn get_struct_mut(&mut self, ident: &Ident) -> Option<&mut ItemStruct> {
        self.get_item_mut(ident).and_then(|item| match item {
            Item::Struct(item_struct) => Some(item_struct),
            _ => None,
        })
    }

    /// Get an [`ItemEnum`] from the [`File`] by [`Ident`].
    pub fn get_enum(&self, ident: &Ident) -> Option<&ItemEnum> {
        self.get_item(ident).and_then(|item| match item {
            Item::Enum(item_enum) => Some(item_enum),
            _ => None,
        })
    }

    /// Get a mutable [`ItemEnum`] from the [`File`] by [`Ident`].
    pub fn get_enum_mut(&mut self, ident: &Ident) -> Option<&mut ItemEnum> {
        self.get_item_mut(ident).and_then(|item| match item {
            Item::Enum(item_enum) => Some(item_enum),
            _ => None,
        })
    }
}

impl File {
    /// Create a new [`ItemStruct`] with the given [`Ident`].
    pub fn create_struct(&mut self, ident: &Ident) {
        self.push_struct(ItemStruct {
            attrs: Vec::new(),
            vis: Visibility::Public(Token![pub](Span::call_site())),
            struct_token: Token![struct](Span::call_site()),
            ident: ident.clone(),
            generics: Generics::default(),
            fields: Fields::Unit,
            semi_token: Some(Token![;](Span::call_site())),
        });
    }

    /// Create a new [`ItemEnum`] with the given [`Ident`].
    pub fn create_enum(&mut self, ident: &Ident) {
        self.push_enum(ItemEnum {
            attrs: Vec::new(),
            vis: Visibility::Public(Token![pub](Span::call_site())),
            enum_token: Token![enum](Span::call_site()),
            ident: ident.clone(),
            generics: Generics::default(),
            brace_token: Brace::default(),
            variants: Punctuated::new(),
        });
    }
}

impl File {
    /// Get a [`Field`] from the [`ItemStruct`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] and [`Field`] to retrieve.
    pub fn get_struct_field(&self, state: State<'_, '_>) -> Option<&Field> {
        self.get_struct(state.item).and_then(|item_struct| match &item_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .find(|field| field.ident.as_ref().is_some_and(|ident| ident == state.field)),
            _ => None,
        })
    }

    /// Get a mutable [`Field`] from the [`ItemStruct`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] and [`Field`] to retrieve.
    pub fn get_struct_field_mut(&mut self, state: State<'_, '_>) -> Option<&mut Field> {
        self.get_struct_mut(state.item).and_then(|item_struct| match &mut item_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter_mut()
                .find(|field| field.ident.as_ref().is_some_and(|ident| ident == state.field)),
            _ => None,
        })
    }

    /// Get a [`Variant`] from the [`ItemEnum`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] and [`Variant`] to retrieve.
    pub fn get_enum_variant(&self, state: State<'_, '_>) -> Option<&Variant> {
        self.get_enum(state.item).and_then(|item_enum| {
            item_enum.variants.iter().find(|variant| &variant.ident == state.field)
        })
    }

    /// Get a mutable [`Variant`] from the [`ItemEnum`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] and [`Variant`] to retrieve.
    pub fn get_enum_variant_mut(&mut self, state: State<'_, '_>) -> Option<&mut Variant> {
        self.get_enum_mut(state.item).and_then(|item_enum| {
            item_enum.variants.iter_mut().find(|variant| &variant.ident == state.field)
        })
    }

    /// Push a [`Field`] into the [`ItemStruct`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] to push the [`Field`] into.
    pub fn push_struct_field(&mut self, state: State<'_, '_>, field: Field) -> anyhow::Result<()> {
        if let Some(item_struct) = self.get_struct_mut(state.item) {
            match (field.ident.is_some(), &mut item_struct.fields) {
                (true, Fields::Named(fields)) => fields.named.push(field),
                (false, Fields::Unnamed(fields)) => fields.unnamed.push(field),
                (true, Fields::Unnamed(..)) => {
                    anyhow::bail!("File: Attempted to push a named Field into an unnamed Struct")
                }
                (false, Fields::Named(..)) => {
                    anyhow::bail!("File: Attempted to push an unnamed Field into a named Struct")
                }
                (named, Fields::Unit) => {
                    if named {
                        // Create a new `FieldsNamed` with the field
                        item_struct.fields = Fields::Named(syn::parse_quote!({ #field }));
                        item_struct.semi_token = None;
                    } else {
                        // Create a new `FieldsUnnamed` with the field
                        item_struct.fields = Fields::Unnamed(syn::parse_quote!((#field)));
                        item_struct.semi_token = Some(Token![;](Span::call_site()));
                    }
                }
            }
            Ok(())
        } else {
            anyhow::bail!("File: Attempted to push a Field into a non-existent Struct")
        }
    }

    /// Push a [`Field`] into the [`ItemStruct`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] to push the [`Field`] into.
    pub fn push_struct_field_type(
        &mut self,
        state: State<'_, '_>,
        kind: Type,
    ) -> anyhow::Result<()> {
        let field_name = state.field;
        self.push_struct_field(
            state,
            syn::parse_quote! {
                #field_name: #kind
            },
        )
    }

    /// Push a [`Field`] into the [`ItemStruct`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] to push the [`Field`] into.
    pub fn push_struct_field_str(
        &mut self,
        state: State<'_, '_>,
        kind: &str,
    ) -> anyhow::Result<()> {
        self.push_struct_field_type(state, syn::parse_str(kind)?)
    }

    /// Push a [`Field`] into the [`ItemEnum`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] to push the [`Field`] into.
    pub fn push_enum_variant(
        &mut self,
        state: State<'_, '_>,
        variant: Variant,
    ) -> anyhow::Result<()> {
        if let Some(item_enum) = self.get_enum_mut(state.item) {
            item_enum.variants.push(variant);
            Ok(())
        } else {
            anyhow::bail!("File: Attempted to push a Variant into a non-existent Enum")
        }
    }

    /// Push a [`TokenStream`] into the [`ItemEnum`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] to push the [`Field`] into.
    pub fn push_enum_variant_tokens(
        &mut self,
        state: State<'_, '_>,
        tokens: TokenStream,
    ) -> anyhow::Result<()> {
        self.push_enum_variant(state, syn::parse_quote!(#tokens))
    }

    /// Push a [`Field`] into the [`ItemEnum`] by [`Ident`].
    ///
    /// # Note
    /// Use the [`State`] to specify the [`Item`] to push the [`Field`] into.
    pub fn push_enum_variant_str(
        &mut self,
        state: State<'_, '_>,
        kind: &str,
    ) -> anyhow::Result<()> {
        self.push_enum_variant_tokens(state, syn::parse_str(kind)?)
    }
}

impl File {
    /// Push an [`Attribute`] into the [`Item`] by [`Ident`].
    ///
    /// # Note
    /// This will push the [`Attribute`] into the [`ItemStruct`] or [`ItemEnum`]
    /// specified by the [`State`].
    pub fn push_attr(&mut self, state: State<'_, '_>, attr: Attribute) {
        if let Some(item) = self.get_item_mut(state.item) {
            match item {
                Item::Struct(item_struct) => item_struct.attrs.push(attr),
                Item::Enum(item_enum) => item_enum.attrs.push(attr),
                _ => unreachable!("`File::get_item_mut` returned an unexpected variant"),
            }
        }
    }

    /// Push a set of [`Attribute`]s into the [`Item`] by [`Ident`].
    ///
    /// # Note
    /// This will push the [`Attribute`]s into the [`ItemStruct`] or
    /// [`ItemEnum`] specified by the [`State`].
    pub fn push_attrs(&mut self, state: State<'_, '_>, attrs: impl IntoIterator<Item = Attribute>) {
        attrs.into_iter().for_each(|attr| self.push_attr(state, attr));
    }

    /// Push an [`Attribute`] into the [`Field`] by [`Ident`].
    ///
    /// # Note
    /// This will push the [`Attribute`] into the [`Field`]/[`Variant`] of the
    /// [`ItemStruct`]/[`ItemEnum`], specified by the [`State`].
    pub fn push_attr_tokens(&mut self, state: State<'_, '_>, tokens: TokenStream) {
        self.push_attr(state, syn::parse_quote!(#tokens));
    }

    /// Push an [`Attribute`] into the [`Field`] by [`Ident`].
    ///
    /// # Note
    /// This will push the [`Attribute`] into the
    /// [`Field`]/[`Variant`] of the [`ItemStruct`]/[`ItemEnum`],
    /// specified by the [`State`].
    pub fn push_field_attr(&mut self, state: State<'_, '_>, attr: Attribute) {
        if let Some(item) = self.get_item_mut(state.item) {
            match item {
                Item::Struct(item) => {
                    if let Some(field) = item.fields.iter_mut().find(|field| {
                        field.ident.as_ref().is_some_and(|ident| ident == state.field)
                    }) {
                        field.attrs.push(attr);
                    }
                }
                Item::Enum(item) => {
                    if let Some(variant) =
                        item.variants.iter_mut().find(|variant| &variant.ident == state.field)
                    {
                        variant.attrs.push(attr);
                    }
                }
                _ => unreachable!("`File::get_item_mut` returned an unexpected variant"),
            }
        }
    }

    /// Push an [`Attribute`] into the [`Field`] by [`Ident`].
    ///
    /// # Note
    /// This will push the [`Attribute`] into the [`Field`]/[`Variant`] of the
    /// [`ItemStruct`]/[`ItemEnum`], specified by the [`State`].
    pub fn push_field_attr_tokens(&mut self, state: State<'_, '_>, tokens: TokenStream) {
        self.push_field_attr(state, syn::parse_quote!(#tokens));
    }

    /// Push a set of [`Attribute`]s into the [`Field`] by [`Ident`].
    ///
    /// # Note
    /// This will push the [`Attribute`]s into the [`Field`]/[`Variant`] of the
    /// [`ItemStruct`]/[`ItemEnum`], specified by the [`State`].
    pub fn push_field_attrs(
        &mut self,
        state: State<'_, '_>,
        attrs: impl IntoIterator<Item = Attribute>,
    ) {
        attrs.into_iter().for_each(|attr| self.push_field_attr(state, attr));
    }
}
