use convert_case::{Case, Casing};
use froglight_parse::{
    file::{blocks::BlockSpecificationState, VersionBlocks},
    Version,
};
use hashbrown::HashSet;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{File, Ident, Item};

/// A block generator.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockGenerator;

impl BlockGenerator {
    /// Generate blocks from the given [`VersionBlocks`].
    ///
    /// Returns a tuple of two files, one for attributes and one for blocks.
    #[must_use]
    pub fn generate_blocks(version: &Version, blocks: &VersionBlocks) -> (File, File) {
        let (attributes, changes) = Self::generate_attributes(&Self::attribute_list(blocks));
        let attrib_file = File { shebang: None, attrs: Vec::new(), items: attributes };

        let (blocks, _attributes) = Self::create_blocks(version, blocks, &changes);
        let block_file = File { shebang: None, attrs: Vec::new(), items: blocks };

        (attrib_file, block_file)
    }

    fn create_blocks(
        version: &Version,
        blocks: &VersionBlocks,
        overwrite: &[(String, String)],
    ) -> (Vec<Item>, Vec<TokenStream>) {
        let mut items = Vec::with_capacity(blocks.len());
        let mut attributes = Vec::new();

        let version_string = version.to_long_string().replace(['.'], "_");
        let version_ident = Ident::new(&format!("V{version_string}"), Span::call_site());

        for block in blocks.iter() {
            // Generate the block name
            let block_name = block.display_name.to_case(Case::Pascal).replace(['\''], "_");
            let block_name = Ident::new(&block_name, Span::call_site());

            // Generate the block attributes
            let mut block_attributes = TokenStream::new();
            for (index, state) in block.states.iter().enumerate() {
                let mut field_type = Self::attribute_item_name(state);
                if let Some((_, new_ident)) = overwrite.iter().find(|(old, _)| *old == field_type) {
                    field_type = new_ident.clone();
                }
                if index == block.states.len() - 1 {
                    block_attributes.extend(field_type.to_token_stream());
                } else {
                    block_attributes.extend(quote! { #field_type, });
                }
            }

            // Generate the block structs
            if block_attributes.is_empty() {
                items.push(Item::Struct(syn::parse_quote! {
                    pub struct #block_name;
                }));
            } else {
                items.push(Item::Struct(syn::parse_quote! {
                    pub struct #block_name(u16);
                }));
                items.push(Item::Impl(syn::parse_quote! {
                    impl BlockState<#version_ident> for #block_name {
                        type Attributes = (#block_attributes);
                    }
                }));
            }
            attributes.push(block_attributes);
        }

        (items, attributes)
    }
}

impl BlockGenerator {
    /// Generate attributes for the given [`VersionBlocks`].
    ///
    /// Also returns a list of changes made to shorten the attribute names.
    #[must_use]
    pub fn generate_attributes(
        attributes: &[(String, &BlockSpecificationState)],
    ) -> (Vec<Item>, Vec<(String, String)>) {
        let mut items = Vec::new();

        // Generate a struct or enum for each attribute
        for (name, attrib) in attributes {
            let name = Ident::new(name, Span::call_site());
            let item: Item = match attrib {
                BlockSpecificationState::Bool { .. } => Item::Struct(syn::parse_quote! {
                    pub struct #name(pub bool);
                }),
                BlockSpecificationState::Enum { values, .. } => {
                    let values: Vec<_> = values
                        .iter()
                        .map(|v| Ident::new(&v.to_case(Case::Pascal), Span::call_site()))
                        .collect();

                    Item::Enum(syn::parse_quote! {
                        pub enum #name {
                            #(#values,)*
                        }
                    })
                }
                BlockSpecificationState::Int { values, .. } => {
                    let values: Vec<_> = values
                        .iter()
                        .map(|v| Ident::new(&format!("_{v}"), Span::call_site()))
                        .collect();

                    Item::Enum(syn::parse_quote! {
                        pub enum #name {
                            #(#values,)*
                        }
                    })
                }
            };
            items.push(item);
        }

        // Shorten the names of the attributes where possible
        let mut changes = Vec::new();
        let enums = Self::shorten_attribute_names("EnumAttribute", &mut items);
        changes.extend(enums.into_iter().map(|(old, new, _)| (old, new)));
        let intranges = Self::shorten_attribute_names("IntRangeAttribute", &mut items);
        changes.extend(intranges.into_iter().map(|(old, new, _)| (old, new)));
        let intlists = Self::shorten_attribute_names("IntListAttribute", &mut items);
        changes.extend(intlists.into_iter().map(|(old, new, _)| (old, new)));

        (items, changes)
    }

    /// Generate a list of block attributes from the given [`VersionBlocks`].
    #[must_use]
    pub fn attribute_list(blocks: &VersionBlocks) -> Vec<(String, &BlockSpecificationState)> {
        // Collect all unique attributes
        let mut attrib = HashSet::new();
        for block in blocks.iter() {
            for state in &block.states {
                attrib.insert(state);
            }
        }

        // Generate a name for each attribute and sort them
        let mut attrib: Vec<(String, _)> =
            attrib.into_iter().map(|state| (Self::attribute_item_name(state), state)).collect();
        attrib.sort_by(|(a, _), (b, _)| a.cmp(b));

        attrib
    }

    /// Shorten names for unique enum attributes
    ///
    /// Returns a list of shortened attributes in the form of
    /// `(old_ident, new_ident, index)`.
    #[must_use]
    pub fn shorten_attribute_names(
        attrib_type: &str,
        items: &mut [Item],
    ) -> Vec<(String, String, usize)> {
        // Collect all attributes to be shortened
        let mut shortened = Vec::new();
        for (index, item) in items.iter().enumerate() {
            if let Item::Enum(item) = item {
                // If the item is an enum attribute,
                if let Some((attrib_name, _)) = item.ident.to_string().split_once(attrib_type) {
                    // If no other enum attribute starts with the same name,
                    if items.iter().fold(0usize, |counter, item| {
                        if let Item::Enum(item) = item {
                            counter + usize::from(item.ident.to_string().starts_with(attrib_name))
                        } else {
                            counter
                        }
                    }) == 1
                    {
                        shortened.push((
                            item.ident.to_string(),
                            format!("{attrib_name}{attrib_type}"),
                            index,
                        ));
                    }
                }
            }
        }

        // Shorten the names of the attributes
        for (_, new_ident, index) in &shortened {
            if let Item::Enum(item) = &mut items[*index] {
                item.ident = Ident::new(new_ident, Span::call_site());
            }
        }

        // Return the shortened attributes list
        shortened
    }

    /// Generate an item name for the given [`BlockSpecificationState`].
    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    pub fn attribute_item_name(state: &BlockSpecificationState) -> String {
        match state {
            // Return `{name}BooleanAttribute` for boolean states.
            BlockSpecificationState::Bool { name, .. } => {
                let mut name = name.to_case(Case::Pascal);
                name.push_str("BooleanAttribute");
                name
            }
            // Return `{name}Enum{values}Attribute` for enum states.
            BlockSpecificationState::Enum { name, values, .. } => {
                let mut name = name.to_case(Case::Pascal);
                name.push_str("EnumAttribute");

                for value in values {
                    name.push('_');
                    name.push_str(&value.to_case(Case::Pascal));
                }

                name
            }
            // Return `{name}IntRangeAttribute{min}{max}` for integer ranges
            // and `{name}IntListAttribute{values}` for integer lists.
            BlockSpecificationState::Int { name, values, .. } => {
                let mut ints: Vec<i32> =
                    values.iter().map(|value| value.parse().unwrap()).collect();
                ints.sort_unstable();

                let consecutive = ints.windows(2).all(|w| w[0] + 1 == w[1]);
                if consecutive {
                    let mut name = name.to_case(Case::Pascal);
                    name.push_str("IntRangeAttribute");

                    name.push('_');
                    name.push_str(&ints[0].to_string());
                    name.push('_');
                    name.push_str(&ints.last().unwrap().to_string());

                    name
                } else {
                    let mut name = name.to_case(Case::Pascal);
                    name.push_str("IntListAttribute");

                    for value in values {
                        name.push('_');
                        name.push_str(value);
                    }

                    name
                }
            }
        }
    }
}
