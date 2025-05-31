use std::collections::HashMap;

use froglight_dependency::{
    container::DependencyContainer, dependency::minecraft::DataGenerator, version::Version,
};
use serde::Deserialize;
use tokio::fs::File;

use super::Registry;

impl Registry {
    /// Retrieves the [`RegistryReport`] for the given version.
    pub(super) async fn get_report(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<RegistryReport> {
        deps.get_or_retrieve::<DataGenerator>().await?;
        deps.scoped_fut::<DataGenerator, anyhow::Result<_>>(
            async |data: &mut DataGenerator, deps| {
                let path = data.get_version(version, deps).await?;

                let file = File::open(path.join("reports/registries.json")).await?.into_std().await;
                let raw: RawRegistryReport = serde_json::from_reader(&file)?;

                Ok(raw.into_report())
            },
        )
        .await
    }
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RegistryReport {
    pub(super) registries: Vec<RegistryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RegistryItem {
    pub(super) name: String,
    pub(super) default: Option<String>,
    pub(super) values: Vec<String>,
}

// -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
struct RawRegistryReport {
    registries: HashMap<String, RawRegistryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RawRegistryItem {
    default: Option<String>,
    entries: HashMap<String, RawRegistryEntry>,
    protocol_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
struct RawRegistryEntry {
    protocol_id: u32,
}

impl RawRegistryReport {
    fn into_report(self) -> RegistryReport {
        let mut registries = Vec::with_capacity(self.registries.len());

        let mut registries_ordered = self.registries.into_iter().collect::<Vec<_>>();
        registries_ordered.sort_by(|a, b| a.1.protocol_id.cmp(&b.1.protocol_id));

        for (name, raw_item) in registries_ordered {
            let mut values_ordered = raw_item.entries.into_iter().collect::<Vec<_>>();
            values_ordered.sort_by(|a, b| a.1.protocol_id.cmp(&b.1.protocol_id));

            let values = values_ordered.into_iter().map(|(k, _)| k).collect::<Vec<_>>();
            registries.push(RegistryItem { name, default: raw_item.default, values });
        }

        RegistryReport { registries }
    }
}
