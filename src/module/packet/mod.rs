#![allow(dead_code)]

use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;

mod packets;
mod states;

#[derive(ExtractModule)]
#[module(function = Packets::generate)]
pub(crate) struct Packets;

impl Packets {
    async fn generate(_version: &Version, _deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-packet");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-packet\" at \"{}\"", directory.display());
        }

        // let _packets = Self::extract_packet_classes(version, deps).await?;
        // println!("Packets: {_packets:?}");

        Ok(())
    }
}
