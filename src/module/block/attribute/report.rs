use std::{collections::HashMap, ops::RangeInclusive, path::Path};

use froglight_dependency::{
    container::{Dependency, DependencyContainer},
    dependency::minecraft::DataGenerator,
    version::Version,
};
use indexmap::IndexMap;
use serde::Deserialize;

/// A collection of [`ParsedBlockReport`]s.
#[derive(Default, Dependency)]
pub(crate) struct BlockReports(HashMap<Version, ParsedBlockReport>);

impl BlockReports {
    /// Get the [`ParsedBlockReport`] for the given version.
    ///
    /// Returns `None` if the report does not already exist.
    #[inline]
    #[must_use]
    pub(crate) fn version(&self, version: &Version) -> Option<&ParsedBlockReport> {
        self.0.get(version)
    }

    /// Retrieve the [`ParsedBlockReport`] for the given version.
    ///
    /// # Errors
    /// Returns an error if the report could not be retrieved.
    pub(crate) async fn get_version(
        &mut self,
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<&ParsedBlockReport> {
        if !self.0.contains_key(version) {
            deps.get_or_retrieve::<DataGenerator>().await?;
            deps.scoped_fut::<DataGenerator, anyhow::Result<()>>(
                async |data: &mut DataGenerator, deps| {
                    let path = data.get_version(version, deps).await?;
                    self.0.insert(version.clone(), Self::parse_report(path).await?);
                    Ok(())
                },
            )
            .await?;
        }

        Ok(self.version(version).unwrap())
    }

    async fn parse_report(path: &Path) -> anyhow::Result<ParsedBlockReport> {
        let path = path.join("reports/blocks.json");
        tracing::debug!("Parsing \"{}\"", path.display());

        let contents = tokio::fs::read_to_string(path).await?;
        serde_json::from_str(&contents).map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub(crate) struct ParsedBlockReport(pub IndexMap<String, ParsedBlockEntry>);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct ParsedBlockEntry {
    #[serde(default)]
    pub properties: IndexMap<String, Vec<String>>,
    pub states: Vec<ParsedBlockState>,
}

impl ParsedBlockEntry {
    pub(crate) fn default(&self) -> u32 {
        self.states.iter().find(|state| state.default).map_or_else(
            || panic!("ParsedBlockEntry: Unable to find default state!"),
            |state| state.id,
        )
    }

    pub(crate) fn range(&self) -> RangeInclusive<u32> {
        RangeInclusive::new(
            self.states.iter().map(|state| state.id).min().unwrap(),
            self.states.iter().map(|state| state.id).max().unwrap(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub(crate) struct ParsedBlockState {
    pub id: u32,
    #[serde(default)]
    pub default: bool,
}
