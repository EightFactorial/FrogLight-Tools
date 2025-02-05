//! TODO

use std::{path::PathBuf, sync::Once};

use clap::Parser;
use froglight_dependency::{container::SharedDependencies, version::Version};
use tokio::runtime::Builder;

/// The [`main`] command line arguments.
#[derive(Parser)]
pub struct MainArgs {
    /// The version to extract.
    #[clap(short, long)]
    pub version: Version,
    /// The path to the output file.
    ///
    /// If `None`, the result will be logged to the console.
    #[clap(short, long)]
    pub output: Option<PathBuf>,
}

/// The default pre-configured entry point.
///
/// Useful if you only want to log the result to a file or the console.
///
/// # Example
/// ```rust
/// /// Import the function in `main.rs`.
/// pub use froglight_extract::main;
/// ```
#[expect(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn main() -> anyhow::Result<()> {
    #[cfg(feature = "logging")]
    logging();

    let args = MainArgs::parse();

    let runtime =
        Builder::new_multi_thread().enable_all().build().expect("Failed building the Runtime");
    runtime.block_on(extract(args.version, args.output, SharedDependencies::default()))
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
#[allow(clippy::missing_errors_doc, clippy::unused_async)]
pub async fn extract(
    _version: Version,
    _output: Option<PathBuf>,
    _deps: SharedDependencies,
) -> anyhow::Result<()> {
    Ok(())
}

/// Initialize logging with the default environment filter.
///
/// This function will only ever run once, even if called multiple times.
///
/// See [`EnvFilter::from_default_env`](tracing_subscriber::EnvFilter::from_default_env)
/// for more information.
#[cfg(feature = "logging")]
pub fn logging() {
    use tracing_subscriber::fmt;

    static LOGGING: Once = Once::new();
    LOGGING.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::from_default_env();
        fmt().with_env_filter(filter).with_writer(std::io::stderr).init();
    });
}
