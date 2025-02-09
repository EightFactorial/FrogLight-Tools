#![allow(clippy::module_inception, dead_code)]

use std::collections::HashSet;

use attribute::BlockAttributeAttribute;
use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    version::Version,
};
use froglight_extract::module::ExtractModule;

mod attribute;
pub(crate) use attribute::BlockAttributeData;

mod report;
pub(crate) use report::BlockReports;

use crate::ToolConfig;

#[derive(Dependency)]
#[dep(retrieve = BlockAttributes::generate)]
pub(crate) struct BlockAttributes(pub HashSet<BlockAttributeAttribute>);

impl BlockAttributes {
    // TODO: Create an enum representation that can get retrieved from
    // `BlockAttributes` with properly formatted values.

    /// Iterate over all versions and add all unique attributes to the set.
    async fn generate(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
        let mut attributes = HashSet::new();

        deps.get_or_retrieve::<BlockReports>().await?;
        deps.scoped_fut::<BlockReports, anyhow::Result<()>>(
            async |reports: &mut BlockReports, deps| {
                for version in deps.get::<ToolConfig>().unwrap().versions.clone() {
                    for (name, entry) in &reports.get_version(&version, deps).await?.0 {
                        attributes.extend(BlockAttributeData::from_parsed(name, entry).attributes);
                    }
                }
                Ok(())
            },
        )
        .await?;

        Ok(Self(attributes))
    }
}
