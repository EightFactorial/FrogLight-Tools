use std::{fs::OpenOptions, io::Write, path::Path};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{parse_quote, File, Item};
use tracing::info;

use super::structures::ProtocolState;
use crate::{
    config::SupportedVersion,
    modules::util::{struct_name, write_formatted, DataBundle, GIT_HASH},
};

pub(super) fn generate(
    name: &str,
    state: &ProtocolState,
    version: &SupportedVersion,
    path: &Path,
    bundle: &DataBundle,
) -> anyhow::Result<()> {
    let state_name = name.to_case(Case::Pascal);
    let state_ident = Ident::new(&state_name, Span::call_site());
    let version_ident = struct_name(&version.base_version);

    // Store `mod {PACKET};` and `pub use {PACKET}::{PACKET};`
    // items for the `mod.rs`
    let mut file_items = vec![
        Item::Verbatim(TokenStream::new()),
        parse_quote!(
            use froglight_macros::frog_state;
        ),
        Item::Verbatim(TokenStream::new()),
    ];

    // Generate `version/{VERSION}/{STATE}/{PACKET}.rs` modules
    let mut clientbound_idents = Vec::new();
    generate_packet_files(
        &state.clientbound,
        &mut file_items,
        &mut clientbound_idents,
        path,
        bundle,
    )?;
    let clientbound_tokens = generate_macro_state(&clientbound_idents);

    let mut serverbound_idents = Vec::new();
    generate_packet_files(
        &state.serverbound,
        &mut file_items,
        &mut serverbound_idents,
        path,
        bundle,
    )?;
    let serverbound_tokens = generate_macro_state(&serverbound_idents);

    let path = path.join("mod.rs");
    if path.exists() {
        // Trim the path to the last three directories
        let path = path.display().to_string();
        let count = path.chars().filter(|c| *c == '/').count();
        let trimmed = path.split('/').skip(count - 3).collect::<Vec<&str>>().join("/");
        info!("Skipping `{trimmed}` as it already exists");
        return Ok(());
    }

    // Generate `version/{VERSION}/{STATE}/mod.rs`
    //

    // Get the documentation for the mod.rs file
    let mut mod_doc = MOD_DOC
        .replace("{STATE}", &state_name)
        .replace("{VERSION}", &version.base_version.to_string());
    mod_doc.push_str(&format!("{GIT_HASH}`"));

    // Write the mod.rs file
    write_formatted(
        &File {
            shebang: None,
            attrs: vec![parse_quote!(#![doc = #mod_doc]), parse_quote!(#![allow(missing_docs)])],
            items: file_items,
        },
        &path,
        bundle,
    )?;

    // Output the `frog_state!` macro with manual formatting
    let mut options = OpenOptions::new().append(true).open(path).unwrap();

    options.write_all("\nfrog_state! {\n\t".as_bytes())?;

    options.write_all(state_ident.to_string().as_bytes())?;
    options.write_all(",\n\t".as_bytes())?;

    options.write_all(version_ident.to_string().as_bytes())?;
    options.write_all(",\n\t".as_bytes())?;

    if !clientbound_tokens.is_empty() {
        options.write_all("Clientbound {\n\t\t".as_bytes())?;
        options.write_all(
            clientbound_tokens
                .to_string()
                .replace(" ,", ",\n\t\t")
                .replace("\t ", "\t")
                .trim_end_matches("\n\t\t")
                .as_bytes(),
        )?;
        options.write_all("\n\t},\n\t".as_bytes())?;
    }

    if !serverbound_tokens.is_empty() {
        options.write_all("Serverbound {\n\t\t".as_bytes())?;
        options.write_all(
            serverbound_tokens
                .to_string()
                .replace(" ,", ",\n\t\t")
                .replace("\t ", "\t")
                .trim_end_matches("\n\t\t")
                .as_bytes(),
        )?;
        options.write_all("\n\t},\n}\n".as_bytes())?;
    }

    Ok(())
}

/// Generate the packet files for a state
///
/// Moved to a function for deduplication
fn generate_packet_files(
    packets: &[String],
    packet_modules: &mut Vec<Item>,
    packet_idents: &mut Vec<Ident>,
    path: &Path,
    bundle: &DataBundle,
) -> anyhow::Result<()> {
    for packet in packets {
        // Get the packet name and module name
        let packet_name = packet.split('/').last().unwrap().replace('$', "");
        packet_idents.push(Ident::new(&packet_name, Span::call_site()));

        let mut packet_file_name = packet_name.to_lowercase();
        let packet_mod = Ident::new(&packet_file_name, Span::call_site());

        // Generate the packet file
        packet_file_name.push_str(".rs");
        super::packets::generate(&packet_name, &path.join(&packet_file_name), bundle)?;

        // Add modules and imports to the `mod.rs`
        packet_modules.push(parse_quote!(mod #packet_mod;));
        packet_modules.push(parse_quote!(
            pub use #packet_mod::*;
        ));
        packet_modules.push(Item::Verbatim(TokenStream::new()));
    }

    Ok(())
}

/// Generate a direction for the `frog_state!` macro
fn generate_macro_state(packet_idents: &[Ident]) -> TokenStream {
    let mut tokens = TokenStream::new();

    for (index, ident) in packet_idents.iter().enumerate() {
        let index = u32::try_from(index).unwrap();
        tokens.extend(quote!(#index => #ident,));
    }

    tokens
}

// fn generate_enum(ident: &Ident, packets: &[String], module_items: &mut
// Vec<Item>) {}

const MOD_DOC: &str = r"[`{STATE}`] state packets for [`{VERSION}`]

@generated by `froglight-generator #";
