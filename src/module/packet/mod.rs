use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;

mod classes;
mod codecs;

#[derive(ExtractModule)]
#[module(function = Packets::generate)]
pub(crate) struct Packets;

impl Packets {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-packet");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-packet\" at \"{}\"", directory.display());
        }

        let _ = Self::extract_packet_codecs(version, deps).await?;

        Ok(())
    }
}
