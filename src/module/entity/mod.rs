use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;

mod effects;
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

        Self::generate_status_effects(deps, &directory).await?;
        Self::generate_status_effect_properties(version, deps, &directory).await?;

        Self::generate_entity_types(deps, &directory).await?;
        Self::generate_entity_type_properties(version, deps, &directory).await?;

        Ok(())
    }
}
