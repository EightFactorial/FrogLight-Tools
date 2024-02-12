use std::sync::Arc;

use proc_macro2::TokenStream;
use syn::{File, Item};

use super::{
    util::{module_name, package_path, write_formatted, GIT_HASH},
    DataBundle, Generate,
};

pub(crate) mod structures;

mod packets;
mod states;
mod versions;

/// A module that generates protocol data.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProtocolModule;

impl Generate for ProtocolModule {
    async fn generate(&self, bundle: Arc<DataBundle>) -> anyhow::Result<()> {
        let mut path = package_path("froglight-protocol", &bundle.workspace)?.to_path_buf();
        path.extend(&["src", "versions"]);

        // Create the versions directory
        if !path.exists() {
            tokio::fs::create_dir_all(&path).await?;
        }

        //  Store `pub mod {VERSION};` items for `version/mod.rs`
        let mut module_items = vec![Item::Verbatim(TokenStream::new())];

        // Generate the {VERSION} modules
        for version in &bundle.config.versions {
            let ident = module_name(&version.version);
            let path = path.join(&ident.to_string());

            // Create the {VERSION} directory
            if !path.exists() {
                tokio::task::spawn_local(tokio::fs::create_dir_all(path.clone()));
            }

            // Add an import to the `mod.rs`
            module_items.push(syn::parse_quote!(pub mod #ident;));

            // Generate the module
            versions::generate(version, &path, &bundle)?;
        }

        // Generate `version/mod.rs`
        //

        // Create the documentation for the mod.rs file
        let mod_doc = format!("{VERSIONS_DOC}{GIT_HASH}`");

        // Write the mod.rs file
        write_formatted(
            &File {
                shebang: None,
                attrs: vec![syn::parse_quote!(#![doc = #mod_doc])],
                items: module_items,
            },
            &path.join("mod.rs"),
            &bundle,
        )
    }
}

const VERSIONS_DOC: &str = r"Protocol versions and version-dependent structs and enums

@generated by `froglight-generator #";