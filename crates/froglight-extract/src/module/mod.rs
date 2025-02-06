//! TODO

use std::{future::Future, pin::Pin};

use froglight_dependency::{container::DependencyContainer, version::Version};
pub use inventory;

mod json;
pub use json::{JsonModule, JsonOutput};

/// A module that can be run by name.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtractModule {
    name: &'static str,
    function: ExtractFn,
}

type ExtractFn = for<'a> fn(
    &'a Version,
    &'a mut DependencyContainer,
) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>;

impl ExtractModule {
    /// Create a new [`ExtractModule`] instance.
    #[inline]
    #[must_use]
    pub const fn new(name: &'static str, function: ExtractFn) -> Self { Self { name, function } }

    /// Get the name of the [`ExtractModule`].
    #[inline]
    #[must_use]
    pub const fn name(&self) -> &'static str { self.name }

    /// Run the [`ExtractModule`] on the given [`DependencyContainer`].
    ///
    /// # Errors
    /// Returns an error if the module fails to run.
    #[inline]
    pub async fn run(
        &self,
        version: &Version,
        container: &mut DependencyContainer,
    ) -> anyhow::Result<()> {
        (self.function)(version, container).await
    }

    /// Collect all registered [`ExtractModule`]s.
    pub fn collect() -> impl Iterator<Item = &'static ExtractModule> {
        inventory::iter::<ExtractModule>.into_iter()
    }
}

inventory::collect!(ExtractModule);
