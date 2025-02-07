//! TODO

use std::{collections::HashMap, future::Future, pin::Pin};

use froglight_dependency::{
    container::{DependencyContainer, SharedDependencies},
    version::Version,
};

/// The extract function.
///
/// Useful if you want to use the extracted data in your own code.
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let version = Version::new_release(1, 21, 4);
///     let dependencies = SharedDependencies::default();
///
///     froglight_extract::extract(version, dependencies.clone()).await?;
///
///     // etc...
/// }
/// ```
#[allow(clippy::missing_errors_doc, clippy::unused_async)]
pub async fn extract(
    version: Version,
    modules: &[String],
    dependencies: SharedDependencies,
) -> anyhow::Result<()> {
    // Collect the `ExtractModule`s into a map.
    let module_map: HashMap<&str, &ExtractModule> =
        ExtractModule::collect().map(|m| (m.name(), m)).collect();

    // Iterate over the specified modules and run them.
    // Reacquire the lock for each module to prevent deadlocks.
    for module in modules {
        if let Some(extract) = module_map.get(module.as_str()) {
            tracing::info!("Running module \"{module}\"");
            extract.run(&version, &mut *dependencies.write().await).await?;
        } else {
            tracing::error!("Unknown module \"{module}\"");
        }
    }

    Ok(())
}

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
