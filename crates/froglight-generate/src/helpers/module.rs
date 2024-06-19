use std::{io::SeekFrom, path::Path};

use proc_macro2::{Span, TokenStream};
use syn::{token::Pub, Ident, Item, Visibility};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
};
use tracing::debug;

use super::{format_file_contents, tokens_to_string};

/// Update the modules in a file.
///
/// `public` determines if the modules should be public or not.
///
/// `reexport` adds a `pub use mod::*` statement for each module.
pub(crate) async fn update_modules(
    path: &Path,
    public: bool,
    reexport: bool,
) -> anyhow::Result<()> {
    debug!("Updating modules for: \"{}\"", path.display());
    update_file_modules(
        &mut OpenOptions::new().read(true).write(true).open(path).await?,
        path,
        public,
        reexport,
    )
    .await
}

/// Update the modules in a file.
///
/// `public` determines if the modules should be public or not.
///
/// `reexport` adds a `pub use mod::*` statement for each module.
///
/// Requires the file to be opened with both
/// [`read`](OpenOptions::read) and [`write`](OpenOptions::write)
/// permissions.
pub(crate) async fn update_file_modules(
    file: &mut File,
    path: &Path,
    public: bool,
    reexport: bool,
) -> anyhow::Result<()> {
    // Read the contents of the file.
    let mut contents = String::new();
    file.seek(SeekFrom::Start(0u64)).await?;
    file.read_to_string(&mut contents).await?;

    // Update the modules
    let updated = inner_update_module(contents, path, public, reexport)?;
    let formatted = format_file_contents(updated).await?;

    // Write the updated contents back to the file.
    file.seek(SeekFrom::Start(0u64)).await?;
    file.write_all(formatted.as_bytes()).await?;
    file.sync_data().await.map_err(Into::into)
}

fn inner_update_module(
    contents: String,
    path: &Path,
    public: bool,
    reexport: bool,
) -> anyhow::Result<String> {
    let mut file = syn::parse_file(&contents)?;

    // Get a list of all the modules in the directory.
    let mut modules = Vec::new();
    for dir in std::fs::read_dir(path.parent().unwrap())? {
        let dir = dir?;
        let dir_path = dir.path();

        // Only include directories and '.rs' files.
        if dir_path.extension().map_or(true, |ext| ext == "rs") {
            let name = dir_path
                .file_name()
                .map(|name| name.to_string_lossy().trim_end_matches(".rs").to_string())
                .unwrap();

            // Skip the `mod.rs` file.
            if name != "mod" {
                // Add the module to the list.
                modules.push(name);
            }
        }
    }

    // Remove the existing modules.
    let index = file.items.iter().position(|item| matches!(item, Item::Mod(_))).unwrap_or_default();
    file.items.retain(|item| !matches!(item, Item::Mod(_)));

    // Add the new modules.
    let mut module_items: Vec<Item> = vec![Item::Verbatim(TokenStream::new())];
    for module in modules {
        let ident = Ident::new(&module, Span::call_site());
        let vis = if public { Visibility::Public(Pub::default()) } else { Visibility::Inherited };

        // Add the module statement.
        module_items.push(syn::parse_quote! {
            #vis mod #ident;
        });

        // If reexporting, add a `pub use mod::*;` statement and a newline.
        if reexport {
            module_items.push(syn::parse_quote! {
                pub use #ident::*;
            });
            module_items.push(Item::Verbatim(TokenStream::new()));
        }
    }
    for module in module_items.into_iter().rev() {
        file.items.insert(index, module);
    }

    // Unparse the file back into a string.
    tokens_to_string(file)
}
