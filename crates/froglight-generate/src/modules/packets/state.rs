use std::path::Path;

use convert_case::{Case, Casing};
use froglight_extract::bundle::ExtractBundle;
use serde_json::Value;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::warn;

use super::Packets;
use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{class_to_struct_name, format_file, update_file_modules, version_struct_name},
};

#[allow(clippy::unused_async)]
impl Packets {
    pub(super) async fn create_state(
        state: &str,
        state_data: &Value,
        path: &Path,
        generate: &GenerateBundle<'_>,
        extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        let state_path = path.join(state);
        if !state_path.exists() {
            warn!("Creating state at \"{}\"", state_path.display());
            tokio::fs::create_dir(&state_path).await?;
        }

        // TODO: Create the packet structs
        let mut clientbound = Vec::new();
        if let Some(clientbound_data) = state_data["clientbound"].as_object() {
            for packet in clientbound_data.values() {
                let name = packet["class"].as_str().unwrap();

                // TODO: Check if the packet matches a previously generated packet
                // clientbound.push(PacketType::Existing { name, path:
                // String::from("crate::version::{VERSION}::path::to::Packet") });

                // Generate a new packet
                Self::create_packet(name, packet, &state_path, generate, extract).await?;
                clientbound.push(PacketType::New { name: class_to_struct_name(name) });
            }
        }

        let mut serverbound = Vec::new();
        if let Some(serverbound_data) = state_data["serverbound"].as_object() {
            for packet in serverbound_data.values() {
                let name = packet["class"].as_str().unwrap();

                // TODO: Check if the packet matches a previously generated packet
                // serverbound.push(PacketType::Existing { name, path:
                // String::from("crate::version::{VERSION}::path::to::Packet") });

                // Generate a new packet
                Self::create_packet(name, packet, &state_path, generate, extract).await?;
                serverbound.push(PacketType::New { name: class_to_struct_name(name) });
            }
        }

        Self::create_state_mod(
            state,
            &state_path.join("mod.rs"),
            clientbound,
            serverbound,
            generate,
            extract,
        )
        .await
    }

    const STATE_DOCS: &'static str = r"//! [`{STATE}`](crate::states::{STATE}) state packets for
//! [`{VERSION}`](super::{VERSION})";

    async fn create_state_mod(
        state: &str,
        path: &Path,

        clientbound: Vec<PacketType>,
        serverbound: Vec<PacketType>,

        generate: &GenerateBundle<'_>,
        _extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        let state = state.to_case(Case::Pascal);
        let version = version_struct_name(&generate.version.base).to_string();

        let state_docs = Self::STATE_DOCS.replace("{STATE}", &state).replace("{VERSION}", &version);

        // Output the documentation
        let output = format!(
            r#"{state_docs}
//!
{GENERATE_NOTICE}
#![allow(missing_docs)]
"#
        );
        mod_file.write_all(output.as_bytes()).await?;

        // Update the module list
        update_file_modules(&mut mod_file, path, false, true).await?;

        // Create the state macro
        let mut imports = String::new();
        let clientbound = Self::state_packets(&clientbound, &mut imports);
        let serverbound = Self::state_packets(&serverbound, &mut imports);

        let output = format!(
            r#"
froglight_macros::frog_state! {{
    {state},
    {version},
    Clientbound {{{clientbound}}},
    Serverbound {{{serverbound}}},
}}
    "#
        );
        mod_file.write_all(output.as_bytes()).await?;

        format_file(&mut mod_file).await
    }

    /// Create the macro body for the state packets.
    fn state_packets(packets: &[PacketType], imports: &mut String) -> String {
        let mut output = String::new();

        for (id, packet) in packets.iter().enumerate() {
            let name = match packet {
                PacketType::New { name } => name,
                PacketType::Existing { name, path } => {
                    imports.push_str(&format!("pub use {path}::{name};\n"));
                    name
                }
            };
            output.push_str(&format!("\n        {id}u32 => {name},"));
        }

        if !output.is_empty() {
            output.push_str("\n    ");
        }

        output
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum PacketType {
    New { name: String },
    Existing { name: String, path: String },
}
