#![allow(dead_code)]

use convert_case::{Case, Casing};
use syn::{
    punctuated::Punctuated, token::Brace, Attribute, Field, Fields, FieldsNamed, GenericArgument,
    Ident, Item, PathArguments, PathSegment, Type,
};

/// Structs that require manually defined fields.
const MANUAL_STRUCT_FIELDS: &[(&str, StructAction)] = &[(
    "BlockSet",
    StructAction::Fields(&[
        ("kind", "u32"),
        ("name", "BlockSetName"),
        ("block_ids", "BlockSetBlockIds"),
    ]),
)];

/// Actions to take on a struct.
enum StructAction {
    Replace(&'static str),
    Fields(&'static [(&'static str, &'static str)]),
    Remove,
}

/// Enums that require manually defined variants.
const MANUAL_ENUM_VARIANTS: &[(&str, EnumAction)] =
    &[("PreviousMessagesSignature", EnumAction::Variants(&["Some([u8; 256]) = 0", "None"]))];

/// Actions to take on an enum.
enum EnumAction {
    Replace(&'static str),
    Variants(&'static [&'static str]),
    Remove,
}

/// Field types that should not be modified.
const CORRECT_TYPES: &[&str] =
    &["bool", "u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64", "usize", "isize", "f32", "f64"];

/// Clean up an [`Item`]'s [`Ident`] and [`Fields`].
pub(super) fn process_item(item: Item) -> ProcessResult {
    match item {
        Item::Struct(mut item) => {
            let ident = item.ident.to_string().to_case(Case::Pascal);
            item.ident = Ident::new(&ident, item.ident.span());

            // If the struct has an action, do it
            if let Some((_, action)) = MANUAL_STRUCT_FIELDS.iter().find(|(name, _)| &ident == name)
            {
                match action {
                    // Replace the Item
                    StructAction::Replace(replace) => {
                        ProcessResult::Replaced((*replace).to_string())
                    }
                    // Create the new fields
                    StructAction::Fields(fields) => {
                        let mut named = Punctuated::new();
                        for (name, ty) in *fields {
                            let name = Ident::new(name, item.ident.span());
                            let ty = Ident::new(ty, item.ident.span());
                            named.push(syn::parse_quote!(#name: #ty));
                        }
                        item.fields =
                            Fields::Named(FieldsNamed { brace_token: Brace::default(), named });

                        ProcessResult::Processed(Item::Struct(item))
                    }
                    // Remove the Item
                    StructAction::Remove => ProcessResult::Removed,
                }
            } else {
                // Process the struct's generated fields
                for field in &mut item.fields {
                    process_field(field);
                }

                ProcessResult::Processed(Item::Struct(item))
            }
        }
        Item::Enum(mut item) => {
            let ident = item.ident.to_string().to_case(Case::Pascal);
            item.ident = Ident::new(&ident, item.ident.span());

            // If the enum has an action, do it
            if let Some((_, action)) = MANUAL_ENUM_VARIANTS.iter().find(|(name, _)| &ident == name)
            {
                match action {
                    // Replace the Item
                    EnumAction::Replace(replace) => ProcessResult::Replaced((*replace).to_string()),
                    // Create new variants
                    EnumAction::Variants(variants) => {
                        let mut new_variants = Punctuated::new();
                        for variant in *variants {
                            new_variants.push(syn::parse_str(variant).unwrap());
                        }
                        item.variants = new_variants;

                        ProcessResult::Processed(Item::Enum(item))
                    }
                    // Remove the Item
                    EnumAction::Remove => ProcessResult::Removed,
                }
            } else {
                // Process the enum's generated variants
                for variant in &mut item.variants {
                    let ident = variant.ident.to_string().to_case(Case::Pascal);
                    variant.ident = Ident::new(&ident, variant.ident.span());

                    for field in &mut variant.fields {
                        process_field(field);
                    }
                }

                ProcessResult::Processed(Item::Enum(item))
            }
        }
        _ => ProcessResult::Processed(item),
    }
}

pub(super) enum ProcessResult {
    Replaced(String),
    Processed(Item),
    Removed,
}

fn process_field(field: &mut Field) {
    if let Type::Path(path) = &mut field.ty {
        for segment in &mut path.path.segments {
            if let Some(attr) = process_path(segment) {
                field.attrs.push(attr);
            }
        }
    }
}

fn process_path(segment: &mut PathSegment) -> Option<Attribute> {
    // Check the segment's generic arguments
    let mut returned = None;
    if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
        for arg in &mut args.args {
            if let GenericArgument::Type(Type::Path(path)) = arg {
                for segment in &mut path.path.segments {
                    if let Some(attr) = process_path(segment) {
                        returned = Some(attr);
                    }
                }
            }
        }
    }

    // Check if the segment's type is a varint or varlong
    let segment_type = segment.ident.to_string();
    match segment_type.as_str() {
        "varint" => {
            segment.ident = Ident::new("u32", segment.ident.span());
            return Some(syn::parse_quote!(#[frog(var)]));
        }
        "varlong" => {
            segment.ident = Ident::new("u64", segment.ident.span());
            return Some(syn::parse_quote!(#[frog(var)]));
        }
        _ => {}
    }

    // Format the field type to match rust conventions
    if !CORRECT_TYPES.contains(&segment_type.as_str()) {
        segment.ident =
            Ident::new(&segment.ident.to_string().to_case(Case::Pascal), segment.ident.span());
    }

    returned
}
