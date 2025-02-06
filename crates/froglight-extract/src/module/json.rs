use froglight_dependency::{container::DependencyContainer, version::Version};
use froglight_tool_macros::{Dependency, ExtractModule};

/// A module that extracts data as JSON.
#[derive(Clone, Copy, PartialEq, Eq, Hash, ExtractModule)]
#[module(path = crate, name = "json", function = JsonModule::extract)]
pub struct JsonModule;

/// The output of [`JsonModule`].
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Dependency)]
pub struct JsonOutput(pub serde_json::Value);

impl JsonModule {
    fn extract(_version: &Version, _deps: &mut DependencyContainer) -> anyhow::Result<()> {
        todo!()
    }
}
