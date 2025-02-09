#![expect(unused_imports)]

use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_extract::module::ExtractModule;

mod attribute;
pub(crate) use attribute::{BlockAttributes, BlockReports};

mod property;
// pub(crate) use property::BlockProperties;

#[derive(ExtractModule)]
#[module(function = Blocks::generate)]
pub(crate) struct Blocks;

impl Blocks {
    async fn generate(_version: &Version, deps: &mut DependencyContainer) -> anyhow::Result<()> {
        let attrs = deps.get_or_retrieve::<BlockAttributes>().await?;

        let mut attrs: Vec<_> = attrs.0.iter().collect();
        attrs.sort_unstable_by(|a, b| match a.name.cmp(&b.name) {
            std::cmp::Ordering::Equal => a.values.cmp(&b.values),
            other => other,
        });

        for attr in attrs {
            tracing::info!("Attribute: {attr:?}");
        }

        Ok(())
    }
}
