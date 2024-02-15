use std::path::Path;

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span};
use syn::{parse_quote, File};
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

    let mut items = vec![
        // parse_quote!(
        //     use crate::io::{FrogRead, FrogWrite};
        // ),
        // Item::Verbatim(TokenStream::new()),
    ];

    // Generate the packet struct
    // TODO: Get the packet fields
    // TODO: Automatically derive `Copy` and `PartialEq`/`Eq`/`Hash` where possible
    items.push(parse_quote!(
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct #packet_ident;
    ));

    write_formatted(&File { shebang: None, attrs: Vec::new(), items }, path, bundle)
}
