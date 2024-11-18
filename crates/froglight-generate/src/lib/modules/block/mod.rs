use convert_case::{Case, Casing};
use froglight_parse::file::{blocks::BlockSpecificationState, VersionBlocks};
use hashbrown::HashSet;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{File, Ident, Item};

/// A block generator.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockGenerator;

impl BlockGenerator {
    /// Generate blocks from the given [`VersionBlocks`].
    ///
    /// Returns a tuple of two files, one for attributes and one for blocks.
    #[must_use]
    pub fn generate_blocks(blocks: &VersionBlocks) -> (File, File) {
        let (attributes, changes) = Self::generate_attributes(blocks);
        let blocks = Self::create_blocks(blocks, &changes);

        let attrib_file = File { shebang: None, attrs: Vec::new(), items: attributes };
        let block_file = File { shebang: None, attrs: Vec::new(), items: blocks };

        (attrib_file, block_file)
    }

    fn create_blocks(blocks: &VersionBlocks, changes: &[(Ident, Ident)]) -> Vec<Item> {
        let mut items = Vec::with_capacity(blocks.len());

        for block in blocks.iter() {
            let block_name = Ident::new(&block.name.to_case(Case::Pascal), Span::call_site());

            let mut block_fields = TokenStream::new();
            for state in &block.states {
                let field_name = Ident::new(
                    match state.name().as_str() {
                        "type" => "kind",
                        name => name,
                    },
                    Span::call_site(),
                );

                let mut field_type =
                    &Ident::new(&Self::attribute_item_name(state), Span::call_site());
                if let Some((_, new_ident)) = changes.iter().find(|(old, _)| old == field_type) {
                    field_type = new_ident;
                }

                block_fields.extend(quote! {
                    pub #field_name: #field_type,
                });
            }

            if block_fields.is_empty() {
                items.push(Item::Struct(syn::parse_quote! {
                    pub struct #block_name;
                }));
            } else {
                items.push(Item::Struct(syn::parse_quote! {
                    pub struct #block_name {
                        #block_fields
                    }
                }));
            }
        }

        items
    }
}

impl BlockGenerator {
    /// Generate attributes for the given [`VersionBlocks`].
    ///
    /// Also returns a list of changes made to shorten the attribute names.
    #[must_use]
    pub fn generate_attributes(blocks: &VersionBlocks) -> (Vec<Item>, Vec<(Ident, Ident)>) {
        let mut items = Vec::new();

        // Generate a struct or enum for each attribute
        for (name, attrib) in Self::attribute_list(blocks) {
            let name = Ident::new(&name, Span::call_site());
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
    fn attribute_list(blocks: &VersionBlocks) -> Vec<(String, &BlockSpecificationState)> {
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
    fn shorten_attribute_names(
        attrib_type: &str,
        items: &mut [Item],
    ) -> Vec<(Ident, Ident, usize)> {
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
                            item.ident.clone(),
                            Ident::new(&format!("{attrib_name}{attrib_type}"), Span::call_site()),
                            index,
                        ));
                    }
                }
            }
        }

        // Shorten the names of the attributes
        for (_, new_ident, index) in &shortened {
            if let Item::Enum(item) = &mut items[*index] {
                item.ident = new_ident.clone();
            }
        }

        // Return the shortened attributes list
        shortened
    }

    /// Generate an item name for the given [`BlockSpecificationState`].
    fn attribute_item_name(state: &BlockSpecificationState) -> String {
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
