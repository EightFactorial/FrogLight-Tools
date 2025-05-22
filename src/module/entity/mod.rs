use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;

mod types;

#[derive(ExtractModule)]
#[module(function = Entities::generate)]
pub(crate) struct Entities;

impl Entities {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let mut directory = std::env::current_dir()?;
        directory.push("crates/froglight-entity");

        if !tokio::fs::try_exists(&directory).await? {
            anyhow::bail!("Could not find \"froglight-entity\" at \"{}\"", directory.display());
        }

        let _entities = Self::extract_entity_types(version, deps).await?;
        println!("Entities: {_entities:#?}");

        Ok(())
    }

    #[expect(dead_code, unused_variables)]
    async fn generate_module(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
