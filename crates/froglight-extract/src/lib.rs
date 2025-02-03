#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use tokio::runtime::Builder;

#[cfg(feature = "logging")]
pub mod log;

/// The default pre-configured entry point.
///
/// Useful if you only want to log the result to the console or a file.
///
/// # Example
/// ```rust
/// /// Import the function in `main.rs`.
/// pub use froglight_extract::main;
/// ```
#[expect(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn main() -> anyhow::Result<()> {
    let runtime =
        Builder::new_multi_thread().enable_all().build().expect("Failed building the Runtime");
    runtime.block_on(extract())
}

/// The extract function.
///
/// Useful if you want to use the extracted data in your own code.
///
/// ```rust
/// #[tokio::main]
/// async fn main() {
///     let data = froglight_extract::extract().await;
///     // etc...
/// }
/// ```
#[expect(clippy::missing_errors_doc, clippy::unused_async)]
pub async fn extract() -> anyhow::Result<()> {
    #[cfg(feature = "logging")]
    log::init();

    Ok(())
}
