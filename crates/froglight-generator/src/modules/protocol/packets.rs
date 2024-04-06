use std::path::Path;

use convert_case::{Case, Casing};
use froglight_data::Version;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde_json::{Map, Value};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token::Brace,
    Attribute, Field, Fields, FieldsNamed, File, Generics, Item, ItemStruct, Token, Visibility,
};
use tracing::debug;

use crate::modules::util::{write_formatted, DataBundle};

pub(super) fn generate(
    full_name: &str,
    name: &str,
    version: &Version,
    path: &Path,
    bundle: &DataBundle,
) -> anyhow::Result<()> {
    if path.exists() {
        // Trim the path to the last three directories
        let path = path.display().to_string();
        let count = path.chars().filter(|c| *c == '/').count();
        let trimmed = path.split('/').skip(count - 3).collect::<Vec<&str>>().join("/");
        debug!("Skipping `{trimmed}` as it already exists");
        return Ok(());
    }

    // Import the FrogReadWrite macro
    let mut items = vec![
        Item::Verbatim(TokenStream::new()),
        Item::Use(parse_quote! {
                    use froglight_macros::FrogReadWrite;
        }),
        Item::Verbatim(TokenStream::new()),
    ];

    // Get the packet data
    let (_, data) = bundle.version_data.get(version).unwrap();
    let packet_data = &data["protocol"]["fields"][full_name];

    // Get the fields for the packet
    let (item_fields, packet_fields) = match packet_data {
        Value::Object(data) => fields_with_names(data),
        Value::Array(data) => fields_without_names(data.as_slice()),
        Value::Null => (Fields::Unit, Vec::new()),
        other => anyhow::bail!("Invalid packet data for `{full_name}`: {other:?}"),
    };

    // Get packet struct attributes
    let attrs = get_derives(&packet_fields);

    // Manually create the packet struct
    items.push(Item::Struct(ItemStruct {
        attrs,
        vis: Visibility::Public(Token![pub](Span::call_site())),
        struct_token: Token![struct](Span::call_site()),
        ident: Ident::new(&name.to_case(Case::Pascal), Span::call_site()),
        generics: Generics::default(),
        semi_token: if item_fields.is_empty() || packet_fields.is_empty() {
            Some(Token![;](Span::call_site()))
        } else {
            None
        },
        fields: item_fields,
    }));

    // Write the file
    write_formatted(&File { shebang: None, attrs: Vec::new(), items }, path, bundle)
}

/// Get the fields for the packet struct
fn fields_with_names(fields: &Map<String, Value>) -> (Fields, Vec<String>) {
    let mut punctuated = Punctuated::new();
    let mut field_names = Vec::new();

    for (name, _value) in fields {
        let mut name = name.clone();
        if name == "type" {
            name = "kind".to_string();
        }

        let field_ident = Ident::new(&name.to_case(Case::Snake), Span::call_site());

        punctuated.push(Field {
            attrs: Vec::new(),
            vis: Visibility::Public(Token![pub](Span::call_site())),
            mutability: syn::FieldMutability::None,
            ident: Some(field_ident.clone()),
            colon_token: None,
            ty: parse_quote!(()),
        });
        field_names.push(field_ident.to_string());
    }

    if field_names.is_empty() {
        (Fields::Unit, field_names)
    } else {
        (
            Fields::Named(FieldsNamed { brace_token: Brace::default(), named: punctuated }),
            field_names,
        )
    }
}

/// Get the fields for the packet struct, while generating field names
fn fields_without_names(fields: &[Value]) -> (Fields, Vec<String>) {
    let mut punctuated = Punctuated::new();
    let mut field_names = Vec::new();

    for (field_index, _value) in fields.iter().enumerate() {
        let field_ident = Ident::new(&format!("field_{field_index}"), Span::call_site());

        punctuated.push(Field {
            attrs: Vec::new(),
            vis: Visibility::Public(Token![pub](Span::call_site())),
            mutability: syn::FieldMutability::None,
            ident: Some(field_ident.clone()),
            colon_token: None,
            ty: parse_quote!(()),
        });
        field_names.push(field_ident.to_string());
    }

    if field_names.is_empty() {
        (Fields::Unit, field_names)
    } else {
        (
            Fields::Named(FieldsNamed { brace_token: Brace::default(), named: punctuated }),
            field_names,
        )
    }
}

/// Determine which derives to use for the packet struct
fn get_derives(fields: &[String]) -> Vec<Attribute> {
    let unique = fields.iter().unique().collect_vec();

    // If there are no fields, we can derive everything
    if unique.is_empty() {
        return syn::parse2::<AttrWrapper>(
            quote! { #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, FrogReadWrite)] },
        )
        .unwrap()
        .into_inner();
    }

    // Check for types that can't be derived
    let mut can_eq = true;
    let mut can_copy = true;
    let mut should_default = true;

    for field_type in unique {
        match field_type.as_str() {
            "f32" | "f64" => {
                can_eq = false;
            }
            "String" | "Vec" => {
                can_copy = false;
                should_default = false;
            }
            _ => {}
        }
    }

    // Add tokens for the derives
    let mut tokens = TokenStream::new();

    tokens.extend(quote!(Debug,));
    if should_default {
        tokens.extend(quote!(Default,));
    }

    tokens.extend(quote!(Clone,));
    if can_copy {
        tokens.extend(quote!(Copy,));
    }
    if can_eq {
        tokens.extend(quote!(PartialEq, Eq, Hash));
    }

    syn::parse2::<AttrWrapper>(quote! { #[derive(#tokens, FrogReadWrite)] }).unwrap().into_inner()
}

/// Wrapper for `Attribute` to allow for parsing
struct AttrWrapper(Vec<Attribute>);

impl AttrWrapper {
    /// Unwrap the inner `Attribute`
    fn into_inner(self) -> Vec<Attribute> { self.0 }
}

impl Parse for AttrWrapper {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AttrWrapper(input.call(Attribute::parse_outer)?))
    }
}
