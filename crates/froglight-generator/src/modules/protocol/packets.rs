use std::path::Path;

use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, File, Item};
use tracing::debug;

use crate::modules::util::{write_formatted, DataBundle};

pub(super) fn generate(name: &str, path: &Path, bundle: &DataBundle) -> anyhow::Result<()> {
    if path.exists() {
        // Trim the path to the last three directories
        let path = path.display().to_string();
        let count = path.chars().filter(|c| *c == '/').count();
        let trimmed = path.split('/').skip(count - 3).collect::<Vec<&str>>().join("/");
        debug!("Skipping `{trimmed}` as it already exists");
        return Ok(());
    }

    let packet_ident = Ident::new(&name.to_case(Case::Pascal), Span::call_site());

    let mut items: Vec<Item> = Vec::new();

    // Get the packet fields
    // let mut fields = Vec::new();

    // Import the FrogReadWrite macro
    items.extend([
        Item::Verbatim(TokenStream::new()),
        parse_quote!(
            use froglight_macros::FrogReadWrite;
        ),
        Item::Verbatim(TokenStream::new()),
    ]);

    // Generate the packet struct
    // TODO: Get the packet fields
    // TODO: Automatically derive `Copy` and `PartialEq`/`Eq`/`Hash` where possible
    items.push(parse_quote!(
        #[derive(Debug, Clone, PartialEq, Eq, Hash, FrogReadWrite)]
        #[cfg_attr(feature = "reflect", derive(bevy_reflect::Reflect))]
        pub struct #packet_ident;
    ));

    write_formatted(&File { shebang: None, attrs: Vec::new(), items }, path, bundle)
}

/// Determine which derives to use for the packet struct
fn _get_derives(fields: &[String]) -> TokenStream {
    let unique = fields.iter().unique().collect_vec();

    // If there are no fields, we can derive everything
    if unique.is_empty() {
        return quote!(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, FrogReadWrite);
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
        tokens.extend(quote!(PartialEq, Eq, Hash,));
    }

    tokens.extend(quote!(FrogReadWrite));
    tokens
}
