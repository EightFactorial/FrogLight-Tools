#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(feature = "logging")]
pub mod log;

/// The default pre-configured entry point.
///
///
/// # Example
/// To use, simply import the `main` function in your `main.rs` file.
///
/// ```rust
/// pub use froglight_tools::main;
/// ```
#[expect(clippy::missing_panics_doc)]
pub fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(async { tools });
}

/// The tools function.
///
/// The same as running [`froglight_tools::main`](main),
/// but without initializing a [`tokio`] runtime.
///
/// ```rust
/// pub use froglight_tools::tools;
///
/// #[tokio::main]
/// async fn main() { tools().await; }
/// ```
pub async fn tools() {
    #[cfg(feature = "logging")]
    log::init();
}
