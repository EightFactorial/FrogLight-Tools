use std::path::Path;

use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;
use structure::DataStructures;

mod structure;

#[derive(ExtractModule)]
#[module(function = Registry::generate)]
pub(crate) struct Registry;

impl Registry {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-registry");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-registry\" at \"{}\"", directory.display());
        }

        deps.get_or_retrieve::<DataStructures>().await?;
        deps.scoped_fut::<DataStructures, anyhow::Result<()>>(
            async |data: &mut DataStructures, deps| {
                let _structures = data.get_version(version, deps).await?;
                // for (name, data) in structures.0.iter().filter(|(p, _)|
                // !p.starts_with("tags")) {     println!("{}: {data:#?}",
                // name.display()); }

                Ok(())
            },
        )
        .await?;

        Self::generate_registries(version, deps, &directory).await?;

        Ok(())
    }
}

impl Registry {
    /// Generate registries.
    #[expect(clippy::unused_async)]
    async fn generate_registries(
        _version: &Version,
        _deps: &mut DependencyContainer,
        _path: &Path,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
