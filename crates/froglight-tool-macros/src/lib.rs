#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

// ------------------- `froglight-dependency` -------------------

#[cfg(feature = "dependency")]
mod dependency;

/// Derive the `Dependency` and `Retrievable` traits for a type.
///
/// # Note
/// If a type implements `Default`, it already implements `Retrievable`!
///
/// # Example
/// ```rust,ignore
/// use froglight_dependency::container::{Dependency, DependencyContainer};
///
/// #[derive(Default, Dependency)]
/// struct MyDefaultDependency;
///
/// #[derive(Dependency)]
/// #[dep(retrieve = MyDependency::retrieve)]
/// struct MyDependency;
///
/// impl MyDependency {
///     async fn retrieve(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
///         todo!()
///     }
/// }
///
/// // |
/// // V
///
/// impl Dependency for MyDefaultDependency {}
///
/// impl Dependency for MyDependency {}
/// impl Retrievable for MyDependency {
///     async fn retrieve(deps: &mut DependencyContainer) -> anyhow::Result<Self> {
///         MyDependency::retrieve(deps).await
///     }
/// }
/// ```
#[cfg(feature = "dependency")]
#[proc_macro_derive(Dependency, attributes(dep))]
pub fn derive_dependency(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    dependency::derive_dependency(input.into()).into()
}
