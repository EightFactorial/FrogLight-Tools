use std::{io::Write, path::Path, process::Stdio};

use convert_case::{Case, Casing};
use froglight_dependency::container::DependencyContainer;
use tokio::{io::AsyncReadExt, process::Command, sync::OnceCell};
use tracing::trace;

use super::Packets;
use crate::{
    module::packet::{codecs::PacketInfo, VersionCodecs},
    ToolConfig,
};

mod module;
mod packet;

impl Packets {
    pub(super) async fn generate_packets(
        deps: &mut DependencyContainer,
        path: &Path,
    ) -> anyhow::Result<()> {
        static ONCE: OnceCell<anyhow::Result<()>> = OnceCell::const_new();
        ONCE.get_or_init(async || {
            for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                let version_dir =
                    path.join(format!("v{}", version.to_long_string().replace('.', "_")));
                // Create the version directory if it does not exist
                if !tokio::fs::try_exists(&version_dir).await? {
                    tokio::fs::create_dir_all(&version_dir).await?;
                }

                // Generate the `mod.rs` file for the version module
                Self::generate_version_module(&version, &version_dir).await?;

                let codecs = deps.get_or_retrieve::<VersionCodecs>().await?.clone();
                let ver_codecs = codecs.version(&version).unwrap();

                // Generate the `mod.rs` and packet files for the state
                for (state, _packets) in ver_codecs.iter() {
                    let state_dir = version_dir.join(state.to_string().to_lowercase());
                    Self::generate_state_module(state, ver_codecs, &version, deps, &state_dir)
                        .await?;
                }
            }

            Ok(())
        })
        .await
        .as_ref()
        .map_or_else(|e| Err(anyhow::anyhow!(e)), |()| Ok(()))
    }
}

/// Get the enum variant name for a packet.
fn packet_variant(packet: &PacketInfo) -> String {
    let mut class = packet.class.split('/').last().unwrap().to_case(Case::Pascal);
    if let Some((head, tail)) = class.split_once('$') {
        let head = head.split_inclusive(char::is_uppercase).take(2).collect::<String>();
        trace!("Packet: Renaming \"{class}\" to ~\"{head}{tail}\"");
        class = head[..head.len().saturating_sub(1)].to_string() + &tail.to_case(Case::Pascal);
    }
    class.replace("S2C", "").replace("C2S", "").trim_end_matches("Packet").to_string()
}

/// Get the struct name for a packet.
fn packet_ident(packet: &PacketInfo, direction: &str) -> String {
    packet_variant(packet) + &direction.to_uppercase() + "Packet"
}

/// Get the file name for a packet.
fn packet_file(id: usize, packet: &str, direction: &str) -> String {
    format!("{direction}_{id:#04x}_{}.rs", packet.trim_start_matches("minecraft:"))
}

/// Call `rustfmt` and write the contents to the specified path.
async fn write_formatted(contents: &str, path: &Path) -> anyhow::Result<()> {
    // Create a `stdin` with the contents to be formatted
    let stdin = {
        let (reader, mut writer) = std::io::pipe()?;
        writer.write_all(contents.as_bytes())?;
        Stdio::from(reader)
    };

    // Call `rustfmt` to format the contents
    let mut cmd = Command::new("rustfmt");
    let mut cmd = cmd.stdin(stdin).stdout(Stdio::piped()).spawn()?;

    // Write the formatted contents to a file
    let mut formatted = String::new();
    cmd.stdout.as_mut().unwrap().read_to_string(&mut formatted).await?;
    tokio::fs::write(path, formatted).await?;

    Ok(())
}
