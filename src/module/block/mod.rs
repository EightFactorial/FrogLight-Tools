use froglight_dependency::{
    container::DependencyContainer, dependency::minecraft::MinecraftCode, version::Version,
};
use froglight_extract::module::ExtractModule;

#[derive(ExtractModule)]
#[module(function = Blocks::generate)]
pub(crate) struct Blocks;

impl Blocks {
    async fn generate(version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        deps.get_or_retrieve::<MinecraftCode>().await?;
        let mut mc_code = deps.take::<MinecraftCode>().unwrap();
        let _code = mc_code.get_version(version, deps).await?;

        deps.insert(mc_code);
        Ok(())
    }
}
