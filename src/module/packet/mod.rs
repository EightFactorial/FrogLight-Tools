use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;

mod classes;
mod generate;

mod codecs;
pub(crate) use codecs::{NetworkCodecs, VersionCodecs};

#[derive(ExtractModule)]
#[module(function = Packets::generate)]
pub(crate) struct Packets;

impl Packets {
    async fn generate(_: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-packet");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-packet\" at \"{}\"", directory.display());
        }

        // Generate the packet structs and implementations
        Self::generate_packets(deps, &directory.join("src")).await?;

        Ok(())
    }
}
